//! Canvas for rendering terminal content to images

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage, imageops::FilterType};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

#[cfg(all(feature = "freetype", not(feature = "swash")))]
use crate::font::render_text;

#[cfg(feature = "swash")]
use crate::font::render_text as render_text_swash;

#[cfg(all(
    feature = "ab_glyph",
    not(feature = "swash"),
    not(feature = "freetype")
))]
use ab_glyph::{Font, FontArc, PxScale};

#[cfg(all(
    feature = "ab_glyph",
    not(feature = "swash"),
    not(feature = "freetype")
))]
use imageproc::drawing::draw_text_mut;

/// Canvas for drawing shapes and text
pub struct Canvas {
    background: Pixmap,
    text_layer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    char_width: u32,
    char_height: u32,
}

impl Canvas {
    /// Create a new canvas with the given dimensions
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let background =
            Pixmap::new(width, height).ok_or_else(|| "Failed to create pixmap".to_string())?;

        let text_layer = ImageBuffer::new(width, height);

        Ok(Self {
            background,
            text_layer,
            char_width: 8,
            char_height: 16,
        })
    }

    /// Set character size for text rendering
    pub fn set_char_size(&mut self, width: u32, height: u32) {
        self.char_width = width;
        self.char_height = height;
    }

    /// Get character width
    #[allow(dead_code)]
    pub fn char_width(&self) -> u32 {
        self.char_width
    }

    /// Get character height
    #[allow(dead_code)]
    pub fn char_height(&self) -> u32 {
        self.char_height
    }

    /// Fill the entire canvas with a color
    pub fn fill(&mut self, color: [u8; 4]) {
        let color = Color::from_rgba8(color[0], color[1], color[2], color[3]);
        self.background.fill(color);
    }

    /// Paint `image` as the canvas background using a "cover" fit: the image is
    /// scaled (preserving aspect ratio) to fully cover the canvas, then the
    /// overflow is center-cropped. Straight-alpha RGBA is written directly into
    /// the pixmap, which is exact for opaque images.
    pub fn fill_background_image(&mut self, image: &DynamicImage) {
        let cw = self.background.width();
        let ch = self.background.height();
        let (iw, ih) = image.dimensions();
        if iw == 0 || ih == 0 {
            return;
        }

        let scale = (cw as f32 / iw as f32).max(ch as f32 / ih as f32);
        let nw = ((iw as f32 * scale).round() as u32).max(1);
        let nh = ((ih as f32 * scale).round() as u32).max(1);

        let rgba = image.to_rgba8();
        let resized = image::imageops::resize(&rgba, nw, nh, FilterType::Triangle);

        let off_x = nw.saturating_sub(cw) / 2;
        let off_y = nh.saturating_sub(ch) / 2;
        let cropped = image::imageops::crop_imm(&resized, off_x, off_y, cw, ch).to_image();

        let src = cropped.into_raw();
        let dst = self.background.data_mut();
        if dst.len() == src.len() {
            dst.copy_from_slice(&src);
        }
    }

    /// Fill a rectangle with a color
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));
            self.background
                .fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    /// Draw a title bar background at the top
    pub fn draw_title_bar(&mut self, _title: &str, _padding: u32) {
        let height = 32;
        let bg = [40, 40, 45, 255];
        self.fill_rect(0, 0, self.width(), height, bg);
        self.fill_rect(0, height as i32 - 2, self.width(), 2, [60, 60, 65, 255]);
    }

    /// Draw text using ab_glyph fonts with per-character fallback
    #[cfg(all(
        feature = "ab_glyph",
        not(feature = "swash"),
        not(feature = "freetype")
    ))]
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: [u8; 4],
        font: &FontArc,
        fallback: &FontArc,
        scale: PxScale,
    ) {
        let rgba = Rgba(color);
        // Grid cell width from the primary font's space advance.
        let units_per_em = font.units_per_em().unwrap_or(2048.0);
        let char_width_px =
            (font.h_advance_unscaled(font.glyph_id(' ')) / units_per_em * scale.x).round() as i32;

        let mut pen_x = x;
        for ch in text.chars() {
            // Primary font when it has the glyph, else fallback (treated as wide).
            let (f, is_wide) = if font.glyph_id(ch).0 != 0 {
                (font, false)
            } else {
                (fallback, true)
            };
            draw_text_mut(
                &mut self.text_layer,
                rgba,
                pen_x,
                y,
                scale,
                f,
                &ch.to_string(),
            );
            pen_x += if is_wide {
                char_width_px * 2
            } else {
                char_width_px
            };
        }
    }

    /// Draw text using FreeType for high-quality rendering
    #[cfg(all(feature = "freetype", not(feature = "swash")))]
    pub fn draw_text_freetype(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: [u8; 4],
        font_size: f32,
    ) {
        let glyphs = render_text(text, font_size);
        let canvas_w = self.text_layer.width() as i32;
        let canvas_h = self.text_layer.height() as i32;

        for (gx, gy, bw, bh, rgba_data) in glyphs {
            let dest_x = x + gx;
            let dest_y = y + gy;

            for row in 0..bh {
                for col in 0..bw {
                    let px = dest_x + col;
                    let py = dest_y + row;

                    if px < 0 || py < 0 || px >= canvas_w || py >= canvas_h {
                        continue;
                    }

                    let src_idx = ((row * bw + col) * 4) as usize;
                    if src_idx + 3 >= rgba_data.len() {
                        continue;
                    }

                    let alpha = rgba_data[src_idx + 3] as u32;
                    if alpha == 0 {
                        continue;
                    }

                    let pixel = self.text_layer.get_pixel_mut(px as u32, py as u32);

                    if alpha == 255 {
                        pixel[0] = color[0];
                        pixel[1] = color[1];
                        pixel[2] = color[2];
                        pixel[3] = 255;
                    } else {
                        // Source-over compositing with straight alpha
                        let dst_a = pixel[3] as u32;
                        let out_a = alpha + ((255 - alpha) * dst_a + 127) / 255;
                        if out_a > 0 {
                            let src_w = alpha * 255;
                            let dst_w = dst_a * (255 - alpha);
                            let total = src_w + dst_w;
                            pixel[0] =
                                ((color[0] as u32 * src_w + pixel[0] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[1] =
                                ((color[1] as u32 * src_w + pixel[1] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[2] =
                                ((color[2] as u32 * src_w + pixel[2] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[3] = out_a as u8;
                        }
                    }
                }
            }
        }
    }

    /// Draw text using swash for rendering
    #[cfg(feature = "swash")]
    pub fn draw_text_swash(&mut self, text: &str, x: i32, y: i32, color: [u8; 4], font_size: f32) {
        let glyphs = render_text_swash(text, font_size);
        let canvas_w = self.text_layer.width() as i32;
        let canvas_h = self.text_layer.height() as i32;

        for (gx, gy, bw, bh, mask) in glyphs {
            let dest_x = x + gx;
            let dest_y = y + gy;

            for row in 0..bh {
                let src_offset = (row * bw) as usize;
                let py = dest_y + row;
                if py < 0 || py >= canvas_h {
                    continue;
                }
                for col in 0..bw {
                    let px = dest_x + col;
                    if px < 0 || px >= canvas_w {
                        continue;
                    }

                    let src_idx = src_offset + col as usize;
                    if src_idx >= mask.len() {
                        continue;
                    }

                    let alpha = mask[src_idx] as u32;
                    if alpha == 0 {
                        continue;
                    }

                    let pixel = self.text_layer.get_pixel_mut(px as u32, py as u32);

                    if alpha == 255 {
                        pixel[0] = color[0];
                        pixel[1] = color[1];
                        pixel[2] = color[2];
                        pixel[3] = 255;
                    } else {
                        // Source-over compositing with straight alpha
                        let dst_a = pixel[3] as u32;
                        let out_a = alpha + ((255 - alpha) * dst_a + 127) / 255;
                        if out_a > 0 {
                            let src_w = alpha * 255;
                            let dst_w = dst_a * (255 - alpha);
                            let total = src_w + dst_w;
                            pixel[0] =
                                ((color[0] as u32 * src_w + pixel[0] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[1] =
                                ((color[1] as u32 * src_w + pixel[1] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[2] =
                                ((color[2] as u32 * src_w + pixel[2] as u32 * dst_w + total / 2)
                                    / total) as u8;
                            pixel[3] = out_a as u8;
                        }
                    }
                }
            }
        }
    }

    /// Get the canvas width
    pub fn width(&self) -> u32 {
        self.background.width()
    }

    /// Get the canvas height
    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.background.height()
    }

    /// Convert the canvas to a final image
    pub fn into_image(self) -> Result<RgbaImage, String> {
        let mut result = RgbaImage::from_raw(
            self.background.width(),
            self.background.height(),
            self.background.data().to_vec(),
        )
        .ok_or_else(|| "Failed to create image from raw data".to_string())?;

        // Composite text layer onto background
        for (x, y, bg_pixel) in result.enumerate_pixels_mut() {
            let text_pixel = self.text_layer.get_pixel(x, y);
            let ta = text_pixel[3] as u32;
            if ta > 0 {
                let inv_ta = 255 - ta;
                bg_pixel[0] =
                    ((text_pixel[0] as u32 * ta + bg_pixel[0] as u32 * inv_ta + 128) >> 8) as u8;
                bg_pixel[1] =
                    ((text_pixel[1] as u32 * ta + bg_pixel[1] as u32 * inv_ta + 128) >> 8) as u8;
                bg_pixel[2] =
                    ((text_pixel[2] as u32 * ta + bg_pixel[2] as u32 * inv_ta + 128) >> 8) as u8;
                bg_pixel[3] = 255;
            }
        }

        Ok(result)
    }
}
