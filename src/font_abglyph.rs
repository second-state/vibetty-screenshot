//! Font loading utilities using ab_glyph

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use std::sync::LazyLock;

/// Global cached font, parsed only once
static FONT: LazyLock<FontArc> = LazyLock::new(|| {
    FontArc::try_from_slice(include_bytes!("../assets/SarasaMonoSC-Light.ttf"))
        .expect("Embedded font is valid")
});

/// Font data container with ab_glyph FontArc
pub struct FontData {
    /// Regular font
    pub font: FontArc,
    /// Font scale for rendering
    pub scale: PxScale,
}

impl FontData {
    /// Create a new FontData with the given font size
    pub fn new(font_size: f32) -> Self {
        Self {
            font: FONT.clone(),
            scale: PxScale::from(font_size),
        }
    }
}

/// Load font with specific size
pub fn load_font_with_size(size: f32) -> Result<FontData, String> {
    Ok(FontData::new(size))
}

/// Get character metrics using ab_glyph
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let font = &*FONT;
    let scale = PxScale::from(font_size);
    let scaled = font.as_scaled(scale);

    let char_width = scaled.h_advance(font.glyph_id(' ')) as u32;
    let char_height = scaled.height() as u32;

    (char_width.max(1), char_height.max(1))
}
