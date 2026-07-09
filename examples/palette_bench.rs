//! Verify palette PNG size with a real Rust implementation (color_quant + png).
//! Compares: image-crate RGB (default), png-crate RGB (best), png-crate palette (best).
use std::collections::HashSet;

use color_quant::NeuQuant;
use image::ImageReader;
use png::{BitDepth, ColorType, Compression, Encoder};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/chensiheng/Downloads/screenshot.png".to_string());

    let img = ImageReader::open(&path).unwrap().decode().unwrap();
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let raw: &[u8] = rgb.as_raw();

    // (1) image crate RGB PNG, default compression — what save_screen_png does today
    let mut baseline = Vec::new();
    rgb.write_to(
        &mut std::io::Cursor::new(&mut baseline),
        image::ImageFormat::Png,
    )
    .unwrap();

    // (2) png crate RGB PNG, best compression
    let mut rgb_best = Vec::new();
    {
        let mut e = Encoder::new(&mut rgb_best, w, h);
        e.set_color(ColorType::Rgb);
        e.set_depth(BitDepth::Eight);
        e.set_compression(Compression::Best);
        let mut wr = e.write_header().unwrap();
        wr.write_image_data(raw).unwrap();
    }

    // (3) png crate palette PNG (NeuQuant 256 + best compression)
    // NOTE: NeuQuant trains on AND indexes RGBA (4 bytes/pixel), not RGB.
    let rgba = img.to_rgba8();
    let rgba_raw: &[u8] = rgba.as_raw();
    let nq = NeuQuant::new(10, 256, rgba_raw);
    let palette: Vec<u8> = nq.color_map_rgb();
    let indices: Vec<u8> = rgba_raw
        .chunks_exact(4)
        .map(|p| nq.index_of(p) as u8)
        .collect();
    let mut pal_best = Vec::new();
    {
        let mut e = Encoder::new(&mut pal_best, w, h);
        e.set_color(ColorType::Indexed);
        e.set_depth(BitDepth::Eight);
        e.set_palette(&palette);
        e.set_compression(Compression::Best);
        let mut wr = e.write_header().unwrap();
        wr.write_image_data(&indices).unwrap();
    }

    let uniq = raw.chunks_exact(3).fold(HashSet::new(), |mut s, p| {
        s.insert([p[0], p[1], p[2]]);
        s
    });

    println!("image: {path}  ({w}x{h}, {} unique colors)", uniq.len());
    println!("original RGBA PNG (vibetty today) :  82.5K  (Pillow-measured, for reference)");
    println!(
        "image-crate RGB PNG (default)     : {:>5.1}K",
        baseline.len() as f64 / 1024.0
    );
    println!(
        "png-crate RGB PNG  (best)         : {:>5.1}K",
        rgb_best.len() as f64 / 1024.0
    );
    println!(
        "png-crate palette  (best, NeuQuant): {:>4.1}K  <-- target",
        pal_best.len() as f64 / 1024.0
    );

    std::fs::write("/tmp/sc_rust_pal.png", &pal_best).unwrap();
    println!("\npalette PNG -> /tmp/sc_rust_pal.png (open to check it's visually lossless)");
}
