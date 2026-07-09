//! Example: render a terminal screen over a custom background image.
//!
//! Pass an image path as the first argument to use that file as the
//! background. With no argument, a synthetic gradient is generated instead.
//!
//! The library itself does no I/O — this example shows how the caller decodes
//! an image and hands a `&DynamicImage` to `capture_screen_with_image`.
//!
//! ```bash
//! cargo run --example bg_image                          # synthetic gradient
//! cargo run --example bg_image -- ~/Pictures/wall.png   # real image
//! ```
use std::io::Write;

use image::{DynamicImage, ImageReader, Rgba, RgbaImage};
use vibetty_screenshot::{ScreenshotConfig, capture_screen_with_image};

/// Build a vivid diagonal gradient so the "cover" fit is visible without a file.
fn gradient_background() -> DynamicImage {
    let (w, h) = (480, 270);
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = (x as f32 / w as f32 * 255.0) as u8;
            let g = (y as f32 / h as f32 * 255.0) as u8;
            let b = ((1.0 - x as f32 / w as f32) * 255.0) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    DynamicImage::ImageRgba8(img)
}

fn main() {
    // Background: a caller-supplied image path, or a generated gradient by default.
    let bg = match std::env::args().nth(1) {
        Some(path) => {
            println!("loading background: {path}");
            ImageReader::open(&path)
                .expect("failed to open background image")
                .with_guessed_format()
                .expect("failed to guess image format")
                .decode()
                .expect("failed to decode background image")
        }
        None => {
            println!("no image path given — using a synthetic gradient");
            gradient_background()
        }
    };

    let mut parser = vt100::Parser::new(6, 44, 0);
    let _ = parser.write(b"\x1b[1;97mCustom Background Image\x1b[0m\r\n");
    let _ = parser.write(b"\x1b[32mHello\x1b[0m rendered over an image\r\n");
    let _ = parser.write("中文测试 老虎\r\n".as_bytes());
    let screen = parser.screen().clone();

    let mut config = ScreenshotConfig::default();
    config.title = Some("bg image".to_string());
    // background_color is ignored when an image is supplied

    let image = capture_screen_with_image(&screen, &config, &bg).expect("render failed");
    let out = "/tmp/bg_test.png";
    image.save(out).expect("save failed");
    println!("saved {out}  ({}x{})", image.width(), image.height());
}
