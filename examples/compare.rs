//! Compare PNG vs JPEG byte sizes for vt100 terminal screenshots.
//!
//! Same `capture_screen` output (RgbaImage) encoded as PNG and as JPEG at
//! several quality levels, across several representative terminal contents.
use std::io::Write;

use image::codecs::jpeg::JpegEncoder;
use vibetty_screenshot::{capture_screen, ScreenshotConfig};

fn feed(p: &mut vt100::Parser, s: &str) {
    let _ = p.write(s.as_bytes());
}

/// Mostly empty: a couple lines of text, rest is the solid background.
fn screen_sparse() -> vt100::Screen {
    let mut p = vt100::Parser::new(24, 80, 0);
    feed(&mut p, "$ ls -la\r\ntotal 0\r\n$ \r\n");
    p.screen().clone()
}

/// Mixed English + ANSI colors + some CJK (like the basic example).
fn screen_ansi() -> vt100::Screen {
    let mut p = vt100::Parser::new(24, 80, 0);
    feed(&mut p, "\x1b[1;32m  vibetty-screenshot\x1b[0m\r\n\r\n");
    feed(&mut p, "  Hello from the terminal!\r\n\r\n");
    feed(&mut p, "  \x1b[33mWarning:\x1b[0m this is an example\r\n");
    feed(&mut p, "  \x1b[31mError:\x1b[0m   something went wrong\r\n");
    feed(&mut p, "  \x1b[34mInfo:\x1b[0m    everything is fine\r\n\r\n");
    feed(&mut p, "  Rendering \x1b[1mbold\x1b[0m and \x1b[2mdim\x1b[0m text.\r\n\r\n");
    feed(&mut p, "  \x1b[36m中文渲染测试\x1b[0m\r\n");
    feed(&mut p, "  你好世界！Hello World!\r\n");
    p.screen().clone()
}

/// Screen packed with CJK glyphs (high-frequency, detailed shapes).
fn screen_cjk() -> vt100::Screen {
    let mut p = vt100::Parser::new(24, 80, 0);
    let lines = [
        "春江潮水连海平海上明月共潮生滟滟随波千万里",
        "何处春江无月明江流宛转绕芳甸月照花林皆似霰",
        "空里流霜不觉飞汀上白沙看不见江天一色无纤尘",
        "皎皎空中孤月轮江畔何人初见月江月何年初照人",
        "人生代代无穷已江月年年望相似不知江月待何人",
        "但见长江送流水白云一片去悠悠青枫浦上不胜愁",
        "谁家今夜扁舟子何处相思明月楼可怜楼上月徘徊",
        "应照离人妆镜台玉户帘中卷不去捣衣砧上拂还来",
        "此时相望不相闻愿逐月华流照君鸿雁长飞光不度",
        "鱼龙潜跃水成文昨夜闲潭梦落花可怜春半不还家",
        "江水流春去欲尽江潭落月复西斜斜月沉沉藏海雾",
        "碣石潇湘无限路不知乘月几人归落月摇情满江树",
        "春江花朝秋月夜往往取酒还独倾人生如梦亦如幻",
        "朝如青丝暮成雪将进酒杯莫停与尔同销万古愁也",
    ];
    for l in lines {
        feed(&mut p, l);
        feed(&mut p, "\r\n");
    }
    p.screen().clone()
}

/// Screen full of solid colored blocks (uniform regions).
fn screen_blocks() -> vt100::Screen {
    let mut p = vt100::Parser::new(24, 80, 0);
    for col in [41, 42, 43, 44, 45, 46, 47, 101, 102, 103, 104, 105, 106, 107, 30, 90] {
        feed(&mut p, &format!("\x1b[{}m{}\x1b[0m\r\n", col, " ".repeat(78)));
    }
    p.screen().clone()
}

fn encode_png(img: &image::RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}

fn encode_jpeg(img: &image::RgbaImage, quality: u8) -> Vec<u8> {
    let (w, h) = (img.width(), img.height());
    let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, quality)
        .encode(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .unwrap();
    buf
}

fn kb(n: usize) -> String {
    format!("{:.1}K", n as f64 / 1024.0)
}

fn main() {
    let config = ScreenshotConfig {
        font_size: 16.0,
        padding: 24,
        show_decorations: true,
        title: Some("compare".to_string()),
        ..ScreenshotConfig::default()
    };

    let cases: Vec<(&str, vt100::Screen)> = vec![
        ("sparse-text", screen_sparse()),
        ("ansi-color", screen_ansi()),
        ("dense-cjk", screen_cjk()),
        ("full-blocks", screen_blocks()),
    ];
    let qualities = [95u8, 85, 75, 50];

    println!(
        "{:<12} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "case", "PNG", "JPG@95", "JPG@85", "JPG@75", "JPG@50"
    );
    println!("{}", "-".repeat(12 + 5 * 9));

    for (name, screen) in &cases {
        let img = capture_screen(screen, &config).unwrap();
        let png = encode_png(&img);
        let jpegs: Vec<usize> = qualities.iter().map(|q| encode_jpeg(&img, *q).len()).collect();

        let (w, h) = (img.width(), img.height());
        println!(
            "{:<12} {:>8} {:>8} {:>8} {:>8} {:>8}   ({w}x{h})",
            name,
            kb(png.len()),
            kb(jpegs[0]),
            kb(jpegs[1]),
            kb(jpegs[2]),
            kb(jpegs[3]),
        );
    }
}
