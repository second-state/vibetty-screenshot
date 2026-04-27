//! Font loading utilities using ab_glyph

use ab_glyph::{Font, FontArc, PxScale};
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
    let units_per_em = font.units_per_em().unwrap_or(2048.0);

    let space_id = font.glyph_id(' ');
    let advance_unscaled = font.h_advance_unscaled(space_id);
    let char_width = ((advance_unscaled / units_per_em) * scale.x).round() as u32;

    let ascent = (font.ascent_unscaled() / units_per_em * scale.y).round() as u32;
    let descent = (font.descent_unscaled() / units_per_em * scale.y).round() as u32;
    let char_height = ascent + descent;

    (char_width.max(1), char_height.max(1))
}
