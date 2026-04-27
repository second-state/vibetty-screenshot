//! Terminal screenshot functionality
//!
//! Renders a vt100::Screen to an image file.
//!
//! # Feature flags
//!
//! - `ab_glyph` (default): Uses ab_glyph/imageproc for pure Rust font rendering
//! - `freetype`: Uses FreeType for high-quality font rendering with hinting
//! - `swash`: Uses swash for pure Rust font rendering with built-in cache

mod canvas;
mod font;
mod theme;
mod utils;

pub use canvas::Canvas;
pub use font::{FontData, get_char_metrics, load_font_with_size};

#[cfg(all(feature = "freetype", not(feature = "swash")))]
pub use font::render_text;

#[cfg(feature = "swash")]
pub use font::render_text;

#[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
pub use ab_glyph::{FontArc, PxScale};

pub use theme::Theme;

use image::ImageError;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ScreenshotError {
    #[error("Failed to load font: {0}")]
    #[allow(dead_code)]
    FontLoadError(String),

    #[error("Canvas error: {0}")]
    CanvasError(String),

    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for screenshot generation
pub struct ScreenshotConfig {
    /// Font size in points
    pub font_size: f32,

    /// Padding around the content (in pixels)
    pub padding: u32,

    /// Background color (R, G, B, A)
    pub background_color: [u8; 4],

    /// Whether to show window decorations
    pub show_decorations: bool,

    /// Window title
    pub title: Option<String>,

    /// Color theme for terminal rendering
    pub theme: Theme,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            padding: 16,
            background_color: [30, 30, 30, 255],
            show_decorations: true,
            title: None,
            theme: Theme::default(),
        }
    }
}

/// Draw text on canvas — delegates to the active backend
fn draw_text(
    canvas: &mut Canvas,
    text: &str,
    x: i32,
    y: i32,
    color: [u8; 4],
    #[allow(unused_variables)] font_size: f32,
    #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
    font: &ab_glyph::FontArc,
    #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
    scale: ab_glyph::PxScale,
) {
    #[cfg(all(feature = "freetype", not(feature = "swash")))]
    {
        canvas.draw_text_freetype(text, x, y, color, font_size);
    }
    #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
    {
        canvas.draw_text_with_font(text, x, y, color, font, scale);
    }
    #[cfg(feature = "swash")]
    {
        canvas.draw_text_swash(text, x, y, color, font_size);
    }
}

/// Capture a vt100::Screen as an image
pub fn capture_screen(
    screen: &vt100::Screen,
    config: &ScreenshotConfig,
) -> Result<image::RgbaImage, ScreenshotError> {
    #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
    let font_data = load_font_with_size(config.font_size)
        .unwrap_or_else(|_| FontData::new(config.font_size));

    #[cfg(feature = "swash")]
    let font_data = load_font_with_size(config.font_size)
        .unwrap_or_else(|_| FontData::new(config.font_size));

    let theme = &config.theme;
    let (char_width, char_height) = get_char_metrics(config.font_size);

    let (rows, cols) = screen.size();
    let padding = config.padding;
    let title_height = if config.show_decorations { 32 } else { 0 };

    // Find the last row with actual content (skip trailing empty rows)
    let mut last_content_row = 0;
    'b: for row in (0..rows).rev() {
        for col in 0..cols {
            if let Some(cell) = screen.cell(row, col)
                && cell.has_contents()
                && cell.contents() != " "
            {
                last_content_row = row;
                break 'b;
            }
        }
    }

    let actual_rows = last_content_row + 1;

    let image_width = cols as u32 * char_width + padding * 2;
    let image_height = actual_rows as u32 * char_height + title_height + padding * 2;

    let mut canvas = Canvas::new(image_width, image_height)
        .map_err(|e| ScreenshotError::CanvasError(e.to_string()))?;

    canvas.set_char_size(char_width, char_height);

    // Fill background
    canvas.fill(config.background_color);

    // Draw title bar if decorations are enabled
    if config.show_decorations {
        let title = config.title.as_deref().unwrap_or("Terminal");
        canvas.draw_title_bar(title, config.padding);

        let title_x = (padding + 8) as i32;
        let title_y = 10;

        #[cfg(all(feature = "freetype", not(feature = "swash")))]
        draw_text(&mut canvas, title, title_x, title_y, [220, 220, 220, 255], config.font_size);

        #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
        draw_text(
            &mut canvas,
            title,
            title_x,
            title_y,
            [220, 220, 220, 255],
            config.font_size,
            &font_data.font,
            font_data.scale,
        );

        #[cfg(feature = "swash")]
        draw_text(&mut canvas, title, title_x, title_y, [220, 220, 220, 255], config.font_size);
    }

    // Draw terminal content
    for row in 0..actual_rows {
        for col in 0..cols {
            if let Some(cell) = screen.cell(row, col) {
                let x = padding + col as u32 * char_width;
                let y = title_height + padding + row as u32 * char_height;

                let bg = cell.bgcolor();
                if bg != vt100::Color::Default {
                    let color = theme.color_to_rgba(bg);
                    let w = if cell.is_wide() {
                        char_width * 2
                    } else {
                        char_width
                    };
                    canvas.fill_rect(x as i32, y as i32, w, char_height, color);
                }

                if cell.has_contents() && !cell.is_wide_continuation() {
                    let fg = cell.fgcolor();
                    let fg_color = theme.get_foreground(fg, cell.bold(), cell.dim());

                    let contents = cell.contents();
                    let w = if cell.is_wide() {
                        char_width * 2
                    } else {
                        char_width
                    };

                    {
                        #[cfg(all(feature = "freetype", not(feature = "swash")))]
                    draw_text(
                        &mut canvas,
                        cell.contents(),
                        x as i32,
                        y as i32,
                        fg_color,
                        config.font_size,
                    );

                    #[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
                    draw_text(
                        &mut canvas,
                        cell.contents(),
                        x as i32,
                        y as i32,
                        fg_color,
                        config.font_size,
                        &font_data.font,
                        font_data.scale,
                    );

                    #[cfg(feature = "swash")]
                    draw_text(
                        &mut canvas,
                        contents,
                        x as i32,
                        y as i32,
                        fg_color,
                        config.font_size,
                    );
                    }
                }
            }
        }
    }

    canvas
        .into_image()
        .map_err(|e| ScreenshotError::CanvasError(e.to_string()))
}

/// Save a vt100::Screen to a PNG file
pub fn save_screen_png(
    screen: &vt100::Screen,
    path: &str,
    config: &ScreenshotConfig,
) -> Result<(), ScreenshotError> {
    let image = capture_screen(screen, config)?;
    image.save(path)?;
    Ok(())
}
