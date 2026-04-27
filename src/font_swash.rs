//! Font loading utilities using swash for pure Rust font rendering

use std::cell::RefCell;
use std::sync::LazyLock;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source};
use swash::FontRef;

/// Embedded font data
static FONT_DATA: &[u8] = include_bytes!("../assets/SarasaMonoSC-Light.ttf");

/// Global font reference, parsed once
static FONT: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    FontRef::from_index(FONT_DATA, 0).expect("Failed to parse embedded font")
});

thread_local! {
    static SCALE_CONTEXT: RefCell<ScaleContext> = RefCell::new(ScaleContext::new());
}

/// Font data container
pub struct FontData {
    /// Font size in pixels
    pub font_size: f32,
}

impl FontData {
    /// Create a new FontData with the given font size
    pub fn new(font_size: f32) -> Self {
        Self { font_size }
    }
}

/// Load font with specific size
pub fn load_font_with_size(size: f32) -> Result<FontData, String> {
    Ok(FontData::new(size))
}

/// Rendered glyph: position and alpha mask data
pub struct GlyphImage {
    /// x offset
    pub x: i32,
    /// y offset
    pub y: i32,
    /// width
    pub w: i32,
    /// height
    pub h: i32,
    /// alpha mask data (single channel)
    pub mask: Vec<u8>,
}

/// Render a string of text using swash and return positioned glyph masks
pub fn render_text(text: &str, font_size: f32) -> Vec<(i32, i32, i32, i32, Vec<u8>)> {
    let font = &*FONT;
    let charmap = font.charmap();
    let glyph_metrics = font.glyph_metrics(&[]).scale(font_size);

    // Compute ascent and line height in pixels
    let font_metrics = font.metrics(&[]);
    let upem = font_metrics.units_per_em as f32;
    let ascent_px = (font_metrics.ascent * font_size / upem) as i32;
    let line_height = ((font_metrics.ascent + font_metrics.descent + font_metrics.leading) * font_size / upem) as i32;

    let mut glyphs = Vec::new();
    let mut pen_x: f32 = 0.0;

    SCALE_CONTEXT.with(|ctx| {
        let mut borrow = ctx.borrow_mut();
        let mut scaler = borrow.builder(*font).size(font_size).hint(true).build();

        for ch in text.chars() {
            let glyph_id = charmap.map(ch);
            let advance = glyph_metrics.advance_width(glyph_id);

            if glyph_id == 0 {
                pen_x += advance;
                continue;
            }

            if let Some(image) = Render::new(&[Source::Outline]).render(&mut scaler, glyph_id) {
                let w = image.placement.width as i32;
                let h = image.placement.height as i32;
                let left = image.placement.left;
                let top = image.placement.top;

                let mask = match image.content {
                    Content::Mask => image.data,
                    Content::Color => {
                        // Color: extract alpha channel
                        let mut alpha = Vec::with_capacity((w * h) as usize);
                        for chunk in image.data.chunks_exact(4) {
                            alpha.push(chunk[3]);
                        }
                        alpha
                    }
                    Content::SubpixelMask => {
                        // Subpixel: average RGB as alpha
                        let mut alpha = Vec::with_capacity((w * h) as usize);
                        for chunk in image.data.chunks_exact(3) {
                            alpha.push(((chunk[0] as u32 + chunk[1] as u32 + chunk[2] as u32) / 3) as u8);
                        }
                        alpha
                    }
                };

                let x_offset = pen_x as i32 + left;
                let y_offset = ascent_px - top;

                // Clip glyph to cell boundaries [0, line_height)
                let skip_top = (-y_offset).max(0) as usize;
                let dest_y = y_offset.max(0);
                let visible_h = (h as usize)
                    .saturating_sub(skip_top)
                    .min((line_height - dest_y) as usize);
                if visible_h == 0 || skip_top as i32 >= h {
                    pen_x += advance;
                    continue;
                }
                let row_bytes = w as usize;
                let clipped_mask = mask[(skip_top * row_bytes)..(skip_top * row_bytes + visible_h * row_bytes)].to_vec();

                glyphs.push((x_offset, dest_y, w, visible_h as i32, clipped_mask));
            }

            pen_x += advance;
        }
    });

    glyphs
}

/// Get character metrics using swash
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let font = &*FONT;
    let charmap = font.charmap();
    let glyph_metrics = font.glyph_metrics(&[]).scale(font_size);
    let space_glyph = charmap.map(' ');
    let char_width = glyph_metrics.advance_width(space_glyph) as u32;

    // Use font metrics (ascent + descent + leading) for proper line height
    let metrics = font.metrics(&[]);
    let upem = metrics.units_per_em as f32;
    let scale_factor = font_size / upem;
    let char_height = ((metrics.ascent + metrics.descent + metrics.leading) * scale_factor) as u32;

    (char_width.max(1), char_height.max(1))
}
