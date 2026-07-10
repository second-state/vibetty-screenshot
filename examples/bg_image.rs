//! Example: render a terminal screen and composite it onto a custom background.
//!
//! The library now renders with a transparent background, so compositing onto
//! an image is just `image::imageops::overlay` on the caller side. Pass an
//! image path as the first argument to use that file as the backdrop; with no
//! argument a synthetic gradient is generated instead.
//!
//! ```bash
//! cargo run --example bg_image                          # synthetic gradient
//! cargo run --example bg_image -- ~/Pictures/wall.png   # real image
//! ```
use std::io::Write;

use image::imageops::{FilterType, crop_imm, overlay, resize};
use image::{ImageReader, Rgba, RgbaImage};
use vibetty_screenshot::{ScreenshotConfig, capture_screen};

/// Build a vivid diagonal gradient fitting `width` x `height`.
fn gradient_background(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = ((1.0 - x as f32 / width as f32) * 255.0) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

/// Scale `src` (preserving aspect ratio) to fully cover `out_w` x `out_h`,
/// then center-crop the overflow — a "cover" fit.
fn cover_fit(src: &RgbaImage, out_w: u32, out_h: u32) -> RgbaImage {
    let (iw, ih) = src.dimensions();
    if iw == 0 || ih == 0 {
        return RgbaImage::new(out_w, out_h);
    }
    let scale = (out_w as f32 / iw as f32).max(out_h as f32 / ih as f32);
    let nw = ((iw as f32 * scale).round() as u32).max(1);
    let nh = ((ih as f32 * scale).round() as u32).max(1);
    let resized = resize(src, nw, nh, FilterType::Triangle);
    let off_x = nw.saturating_sub(out_w) / 2;
    let off_y = nh.saturating_sub(out_h) / 2;
    crop_imm(&resized, off_x, off_y, out_w, out_h).to_image()
}

fn main() {
    let mut parser = vt100::Parser::new(6, 44, 0);
    let _ = parser.write(b"\x1b[1;97mCustom Background Image\x1b[0m\r\n");
    let _ = parser.write(b"\x1b[32mHello\x1b[0m rendered over an image\r\n");
    let _ = parser.write("中文测试 老虎\r\n".as_bytes());
    let screen = parser.screen().clone();

    let mut config = ScreenshotConfig::default();
    config.title = Some("bg image".to_string());

    // Transparent screenshot — nothing but the terminal content (and title bar).
    let shot = capture_screen(&screen, &config).expect("render failed");

    // Build a backdrop exactly the size of the screenshot.
    let mut backdrop = match std::env::args().nth(1) {
        Some(path) => {
            println!("loading background: {path}");
            let img = ImageReader::open(&path)
                .expect("failed to open background image")
                .with_guessed_format()
                .expect("failed to guess image format")
                .decode()
                .expect("failed to decode background image");
            cover_fit(&img.to_rgba8(), shot.width(), shot.height())
        }
        None => {
            println!("no image path given — using a synthetic gradient");
            gradient_background(shot.width(), shot.height())
        }
    };

    // Composite the transparent screenshot on top of the backdrop.
    overlay(&mut backdrop, &shot, 0, 0);

    let out = "/tmp/bg_test.png";
    backdrop.save(out).expect("save failed");
    println!("saved {out}  ({}x{})", backdrop.width(), backdrop.height());
}
