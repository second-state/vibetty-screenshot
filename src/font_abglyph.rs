//! Font loading utilities using ab_glyph
//!
//! Primary font: JetBrains Mono (Latin/ASCII, box-drawing, monospace).
//! Fallback font: Sarasa Mono SC (CJK and any glyph the primary lacks).

use ab_glyph::{Font, FontArc, PxScale};
use std::sync::LazyLock;

/// Embedded primary font data — JetBrains Mono
static FONT_DATA_PRIMARY: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Embedded fallback font data — Sarasa Mono SC
static FONT_DATA_FALLBACK: &[u8] = include_bytes!("../assets/SarasaMonoSC-Light.ttf");

/// Global cached primary font, parsed only once
static FONT_PRIMARY: LazyLock<FontArc> =
    LazyLock::new(|| FontArc::try_from_slice(FONT_DATA_PRIMARY).expect("Primary font is valid"));

/// Global cached fallback font, parsed only once
static FONT_FALLBACK: LazyLock<FontArc> =
    LazyLock::new(|| FontArc::try_from_slice(FONT_DATA_FALLBACK).expect("Fallback font is valid"));

/// Font data container with primary + fallback ab_glyph FontArcs
pub struct FontData {
    /// Primary font (Latin/ASCII/box-drawing)
    pub font: FontArc,
    /// Fallback font (CJK / missing glyphs)
    pub fallback: FontArc,
    /// Font scale for rendering
    pub scale: PxScale,
}

impl FontData {
    /// Create a new FontData with the given font size
    pub fn new(font_size: f32) -> Self {
        Self {
            font: FONT_PRIMARY.clone(),
            fallback: FONT_FALLBACK.clone(),
            scale: PxScale::from(font_size),
        }
    }
}

/// Load font with specific size
pub fn load_font_with_size(size: f32) -> Result<FontData, String> {
    Ok(FontData::new(size))
}

/// Get character metrics using the primary font (defines the 1-cell grid unit)
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let font = &*FONT_PRIMARY;
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
