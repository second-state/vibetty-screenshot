//! Font loading utilities using FreeType for high-quality rendering
//!
//! Primary font: JetBrains Mono (Latin/ASCII, box-drawing, monospace).
//! Fallback font: Sarasa Mono SC (CJK and any glyph the primary lacks).
//! Per-character fallback: a char is rendered with the primary font when it
//! has the glyph, otherwise with the fallback font.

use freetype::face::LoadFlag;
use freetype::Library;
use std::cell::RefCell;
use std::sync::LazyLock;

/// Global FreeType library instance
static LIBRARY: LazyLock<Library> =
    LazyLock::new(|| Library::init().expect("Failed to init FreeType"));

/// Embedded primary font data — JetBrains Mono
static FONT_DATA_PRIMARY: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Embedded fallback font data — Sarasa Mono SC
static FONT_DATA_FALLBACK: &[u8] = include_bytes!("../assets/SarasaMonoSC-Light.ttf");

thread_local! {
    static CACHED_FACE_PRIMARY: RefCell<Option<freetype::Face<&'static [u8]>>> = const { RefCell::new(None) };
    static CACHED_FACE_FALLBACK: RefCell<Option<freetype::Face<&'static [u8]>>> = const { RefCell::new(None) };
}

/// Get or create the cached primary FreeType face (one per thread)
fn get_face_primary() -> freetype::Face<&'static [u8]> {
    CACHED_FACE_PRIMARY.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            *borrow = Some(
                LIBRARY
                    .new_memory_face2(FONT_DATA_PRIMARY, 0)
                    .expect("Failed to load primary font"),
            );
        }
        // Clone uses FT_Reference_Face internally — shares the underlying face
        borrow.as_ref().unwrap().clone()
    })
}

/// Get or create the cached fallback FreeType face (one per thread)
fn get_face_fallback() -> freetype::Face<&'static [u8]> {
    CACHED_FACE_FALLBACK.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            *borrow = Some(
                LIBRARY
                    .new_memory_face2(FONT_DATA_FALLBACK, 0)
                    .expect("Failed to load fallback font"),
            );
        }
        borrow.as_ref().unwrap().clone()
    })
}

/// Font data container with FreeType face
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

/// Render a string of text using FreeType and return positioned glyph bitmaps.
///
/// Each character is rendered with the primary font (JetBrains Mono) when it
/// has the glyph, otherwise with the fallback (Sarasa). The pen advance is
/// snapped to the monospace grid: primary chars advance one cell, fallback
/// chars (assumed wide / CJK) advance two cells and are centered within them.
pub fn render_text(text: &str, font_size: f32) -> Vec<(i32, i32, i32, i32, Vec<u8>)> {
    let face_primary = get_face_primary();
    let face_fallback = get_face_fallback();

    let pixel_size = font_size as u32;
    face_primary
        .set_pixel_sizes(pixel_size, pixel_size)
        .expect("Failed to set pixel size");
    face_fallback
        .set_pixel_sizes(pixel_size, pixel_size)
        .expect("Failed to set pixel size");

    // Grid width from the primary font's space advance.
    face_primary
        .load_char(' ' as usize, LoadFlag::DEFAULT)
        .expect("Failed to load space");
    let char_width_px = (face_primary.glyph().advance().x >> 6) as i32;

    let mut glyphs = Vec::new();
    let mut pen_x: i64 = 0;

    for ch in text.chars() {
        let is_fallback = face_primary.get_char_index(ch as usize).is_none();
        let face = if is_fallback {
            &face_fallback
        } else {
            &face_primary
        };

        face.load_char(ch as usize, LoadFlag::RENDER)
            .expect("Failed to load char");

        let glyph = face.glyph();
        let bitmap = glyph.bitmap();
        let left = glyph.bitmap_left();
        let top = glyph.bitmap_top();
        let bw = bitmap.width();
        let bh = bitmap.rows();
        let pitch = bitmap.pitch();

        let mut rgba_data = Vec::with_capacity((bw * bh * 4) as usize);
        let buf = bitmap.buffer();

        let pixel_mode = bitmap.pixel_mode().unwrap();

        match pixel_mode {
            freetype::bitmap::PixelMode::Gray => {
                for row in 0..bh {
                    for col in 0..bw {
                        let alpha = buf[(row * pitch + col) as usize];
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                    }
                }
            }
            freetype::bitmap::PixelMode::Mono => {
                for row in 0..bh {
                    for col in 0..bw {
                        let byte_idx = (row * pitch + col / 8) as usize;
                        let bit_idx = 7 - (col % 8);
                        let alpha = if byte_idx < buf.len() {
                            ((buf[byte_idx] >> bit_idx) & 1) * 255
                        } else {
                            0
                        };
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                        rgba_data.push(alpha);
                    }
                }
            }
            _ => {}
        }

        // Grid advance: wide (fallback) chars take two cells.
        let advance_px = if is_fallback {
            char_width_px * 2
        } else {
            char_width_px
        };

        // Horizontal placement: primary uses left bearing; fallback centered.
        let x_offset = if is_fallback {
            pen_x as i32 + ((advance_px - bw) / 2).max(0)
        } else {
            pen_x as i32 + left
        };
        let y_offset = -top;

        glyphs.push((x_offset, y_offset, bw, bh, rgba_data));

        pen_x += advance_px as i64;
    }

    glyphs
}

/// Get character metrics using the primary font (defines the 1-cell grid unit)
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let face = get_face_primary();

    let pixel_size = font_size as u32;
    face.set_pixel_sizes(pixel_size, pixel_size)
        .expect("Failed to set pixel size");

    face.load_char(' ' as usize, LoadFlag::DEFAULT)
        .expect("Failed to load space");

    let advance = face.glyph().advance();
    let char_width = (advance.x >> 6) as u32;

    let char_height = (face.height() as u32 * pixel_size) / (face.em_size() as u32).max(1);

    (char_width.max(1), char_height.max(1))
}
