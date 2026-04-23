//! Font loading utilities using FreeType for high-quality rendering

use freetype::face::LoadFlag;
use freetype::Library;
use std::cell::RefCell;
use std::sync::LazyLock;

/// Global FreeType library instance
static LIBRARY: LazyLock<Library> =
    LazyLock::new(|| Library::init().expect("Failed to init FreeType"));

/// Embedded font data
static FONT_DATA: &[u8] = include_bytes!("../assets/SarasaMonoSC-Light.ttf");

thread_local! {
    static CACHED_FACE: RefCell<Option<freetype::Face<&'static [u8]>>> = const { RefCell::new(None) };
}

/// Get or create a cached FreeType face (one per thread)
fn get_face() -> freetype::Face<&'static [u8]> {
    CACHED_FACE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            *borrow = Some(
                LIBRARY
                    .new_memory_face2(FONT_DATA, 0)
                    .expect("Failed to load font"),
            );
        }
        // Clone uses FT_Reference_Face internally — shares the underlying face
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

/// Render a string of text using FreeType and return positioned glyph bitmaps
pub fn render_text(
    text: &str,
    font_size: f32,
) -> Vec<(i32, i32, i32, i32, Vec<u8>)> {
    let face = get_face();

    let pixel_size = font_size as u32;
    face.set_pixel_sizes(pixel_size, pixel_size)
        .expect("Failed to set pixel size");

    let mut glyphs = Vec::new();
    let mut pen_x: i64 = 0;

    for ch in text.chars() {
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

        let x_offset = pen_x as i32 + left;
        let y_offset = -top;

        glyphs.push((x_offset, y_offset, bw, bh, rgba_data));

        let advance = glyph.advance();
        pen_x += advance.x >> 6;
    }

    glyphs
}

/// Get character metrics using FreeType
pub fn get_char_metrics(font_size: f32) -> (u32, u32) {
    let face = get_face();

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
