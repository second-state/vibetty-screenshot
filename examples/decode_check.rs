//! Can the `image` crate decode the png-crate-produced palette PNG?
//! Decode sc_rust_pal.png with `image`, then diff against the original.
use image::ImageReader;

fn main() {
    let pal_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/sc_rust_pal.png".to_string());
    let orig_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "/Users/chensiheng/Downloads/screenshot.png".to_string());

    // (1) decode the palette PNG with the `image` crate
    let pal = ImageReader::open(&pal_path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("image crate 解不开这张 palette PNG");
    println!(
        "✅ image crate 解码成功: color={:?}  {}x{}",
        pal.color(),
        pal.width(),
        pal.height()
    );

    // (2) diff against original to confirm content is intact (within quant error)
    let orig = ImageReader::open(&orig_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();
    let pal_rgb = pal.to_rgb8();
    assert_eq!(orig.dimensions(), pal_rgb.dimensions());

    let total = (orig.width() * orig.height()) as u64;
    let mut diff_pixels = 0u64;
    let mut max_diff = 0u32;
    let mut sum_sq = 0u64;
    for (a, b) in orig.pixels().zip(pal_rgb.pixels()) {
        let d0 = (a[0] as i32 - b[0] as i32).unsigned_abs();
        let d1 = (a[1] as i32 - b[1] as i32).unsigned_abs();
        let d2 = (a[2] as i32 - b[2] as i32).unsigned_abs();
        let m = d0.max(d1).max(d2);
        if m > 0 {
            diff_pixels += 1;
        }
        max_diff = max_diff.max(m);
        sum_sq += (d0 * d0 + d1 * d1 + d2 * d2) as u64;
    }
    let mse = sum_sq as f64 / (total * 3) as f64;
    let psnr = if mse > 0.0 {
        10.0 * (255.0 * 255.0 / mse).log10()
    } else {
        f64::INFINITY
    };

    println!();
    println!("与原图比对:");
    println!(
        "  有差异的像素 : {}/{} ({:.2}%)",
        diff_pixels,
        total,
        100.0 * diff_pixels as f64 / total as f64
    );
    println!("  单通道最大色差: {}", max_diff);
    println!(
        "  PSNR         : {:.2} dB  (>40dB≈视觉无损, >50dB≈几乎不可察觉)",
        psnr
    );
}
