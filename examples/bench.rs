use std::io::Write;
use vibetty_screenshot::{ScreenshotConfig, capture_screen};

fn main() {
    let mut parser = vt100::Parser::new(24, 80, 0);
    let _ = parser.write(b"\x1b[1;32m  vibetty-screenshot\x1b[0m\r\n\r\n");
    let _ = parser.write(b"  Hello from the terminal!\r\n\r\n");
    let _ = parser.write(b"  \x1b[33mWarning:\x1b[0m this is an example\r\n");
    let _ = parser.write(b"  \x1b[31mError:\x1b[0m   something went wrong\r\n");
    let _ = parser.write(b"  \x1b[34mInfo:\x1b[0m    everything is fine\r\n\r\n");
    let _ = parser.write(b"  Rendering \x1b[1mbold\x1b[0m and \x1b[2mdim\x1b[0m text.\r\n\r\n");
    let _ = parser.write("\x1b[36m中文渲染测试\x1b[0m\r\n".as_bytes());
    let _ = parser.write("  Full: \x1b[44m████████\x1b[0m\r\n".as_bytes());

    let screen = parser.screen().clone();
    let config = ScreenshotConfig::default();

    // Warmup
    let _ = capture_screen(&screen, &config);

    let n = 100;
    let start = std::time::Instant::now();
    for _ in 0..n {
        let img = capture_screen(&screen, &config);
        let _ = std::hint::black_box(img);
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed.as_micros() / n as u128;
    println!(
        "capture_screen: {} iterations, {}us/frame ({}us total)",
        n,
        per_frame,
        elapsed.as_micros()
    );

    // Also time PNG save
    let img = capture_screen(&screen, &config).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..n {
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        std::hint::black_box(buf);
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed.as_micros() / n as u128;
    println!(
        "PNG encode:     {} iterations, {}us/frame ({}us total)",
        n,
        per_frame,
        elapsed.as_micros()
    );

    // Full save to disk
    let start = std::time::Instant::now();
    for _ in 0..n {
        img.save("/tmp/bench_out.png").unwrap();
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed.as_micros() / n as u128;
    println!(
        "PNG save disk:  {} iterations, {}us/frame ({}us total)",
        n,
        per_frame,
        elapsed.as_micros()
    );
}
