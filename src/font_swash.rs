//! Font loading utilities using swash for pure Rust font rendering
//!
//! Primary font: JetBrains Mono (Latin/ASCII, box-drawing, monospace).
//! Fallback font: Sarasa Mono SC (CJK and any glyph the primary lacks).
//! Per-character fallback: a char is rendered with the primary font when it
//! has the glyph, otherwise with the fallback font.

use std::cell::RefCell;
use std::sync::LazyLock;
use swash::FontRef;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source};

/// Embedded primary font data — JetBrains Mono
static FONT_DATA_PRIMARY: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Embedded fallback font data — Sarasa Mono SC
static FONT_DATA_FALLBACK: &[u8] = include_bytes!("../assets/SarasaMonoSC-Light.ttf");

/// Global primary font reference, parsed once
static FONT_PRIMARY: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    FontRef::from_index(FONT_DATA_PRIMARY, 0).expect("Failed to parse primary font")
});

/// Global fallback font reference, parsed once
static FONT_FALLBACK: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    FontRef::from_index(FONT_DATA_FALLBACK, 0).expect("Failed to parse fallback font")
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
#[allow(dead_code)]
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

/// Render a string of text using swash and return positioned glyph masks.
///
/// Each character is rendered with the primary font (JetBrains Mono) when it
/// has the glyph, otherwise with the fallback (Sarasa). The pen advance is
/// snapped to the monospace grid: primary chars advance one cell, fallback
/// chars (assumed wide / CJK) advance two cells and are centered within them.
pub fn render_text(text: &str, font_size: f32) -> Vec<(i32, i32, i32, i32, Vec<u8>)> {
    let primary = &*FONT_PRIMARY;
    let fallback = &*FONT_FALLBACK;

    // Grid metrics come from the primary font.
    let primary_metrics = primary.metrics(&[]);
    let upem = primary_metrics.units_per_em as f32;
    let ascent_px = (primary_metrics.ascent * font_size / upem) as i32;
    let line_height = ((primary_metrics.ascent + primary_metrics.descent + primary_metrics.leading)
        * font_size
        / upem) as i32;

    let primary_glyph_metrics = primary.glyph_metrics(&[]).scale(font_size);
    let space_gid = primary.charmap().map(' ');
    let char_width_px = primary_glyph_metrics.advance_width(space_gid) as f32;

    let mut glyphs = Vec::new();
    let mut pen_x: f32 = 0.0;

    SCALE_CONTEXT.with(|ctx| {
        let mut borrow = ctx.borrow_mut();

        for ch in text.chars() {
            let primary_gid = primary.charmap().map(ch);

            // Pick font: primary if it has the glyph, else fallback.
            let (font, gid, is_fallback) = if primary_gid != 0 {
                (primary, primary_gid, false)
            } else {
                let fb_gid = fallback.charmap().map(ch);
                (fallback, fb_gid, true)
            };

            // Grid advance: wide (fallback) chars take two cells.
            let advance_px = if is_fallback {
                char_width_px * 2.0
            } else {
                char_width_px
            };

            if gid == 0 {
                pen_x += advance_px;
                continue;
            }

            // Scaler is bound to a single font; rebuild per char (cheap, and
            // render_text is called per cell which is typically one char).
            let mut scaler = borrow.builder(*font).size(font_size).hint(true).build();
            if let Some(image) = Render::new(&[Source::Outline]).render(&mut scaler, gid) {
                let w = image.placement.width as i32;
                let h = image.placement.height as i32;
                let left = image.placement.left;
                let top = image.placement.top;

                let mask = match image.content {
                    Content::Mask => image.data,
                    Content::Color => {
                        let mut alpha = Vec::with_capacity((w * h) as usize);
                        for chunk in image.data.chunks_exact(4) {
                            alpha.push(chunk[3]);
                        }
                        alpha
                    }
                    Content::SubpixelMask => {
                        let mut alpha = Vec::with_capacity((w * h) as usize);
                        for chunk in image.data.chunks_exact(3) {
                            alpha.push(
                                ((chunk[0] as u32 + chunk[1] as u32 + chunk[2] as u32) / 3) as u8,
                            );
                        }
                        alpha
                    }
                };

                // Horizontal placement. Primary: use the glyph's left bearing.
                // Fallback: center the glyph inside its 2-cell allocation.
                let x_offset = if is_fallback {
                    let cell_w = advance_px as i32;
                    pen_x as i32 + ((cell_w - w) / 2).max(0)
                } else {
                    pen_x as i32 + left
                };
                let y_offset = ascent_px - top;

                // Clip glyph to cell row bounds [0, line_height).
                let skip_top = (-y_offset).max(0) as usize;
                let dest_y = y_offset.max(0);
                let visible_h = (h as usize)
                    .saturating_sub(skip_top)
                    .min((line_height - dest_y) as usize);
                if visible_h == 0 || skip_top as i32 >= h {
                    pen_x += advance_px;
                    continue;
                }
                let row_bytes = w as usize;
                let clipped_mask = mask
                    [(skip_top * row_bytes)..(skip_top * row_bytes + visible_h * row_bytes)]
                    .to_vec();

                glyphs.push((x_offset, dest_y, w, visible_h as i32, clipped_mask));
            }

            pen_x += advance_px;
        }
    });

    glyphs
}

/// Get character metrics using the primary font (defines the 1-cell grid unit)
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let font = &*FONT_PRIMARY;
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
