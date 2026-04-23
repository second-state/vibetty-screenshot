use std::io::Write;
use std::time::Instant;

fn build_screen() -> vt100::Screen {
    let mut parser = vt100::Parser::new(24, 80, 0);
    let _ = parser.write(b"\x1b[1;32m  vibetty-screenshot\x1b[0m\r\n\r\n");
    let _ = parser.write(b"  Hello from the terminal!\r\n\r\n");
    let _ = parser.write(b"  \x1b[33mWarning:\x1b[0m this is an example\r\n");
    let _ = parser.write(b"  \x1b[31mError:\x1b[0m   something went wrong\r\n");
    let _ = parser.write(b"  \x1b[34mInfo:\x1b[0m    everything is fine\r\n\r\n");
    let _ = parser.write(b"  Rendering \x1b[1mbold\x1b[0m and \x1b[2mdim\x1b[0m text.\r\n\r\n");
    let _ = parser.write("  \x1b[36m中文渲染测试\x1b[0m\r\n".as_bytes());
    let _ = parser.write("  你好世界！Hello World!\r\n".as_bytes());
    let _ = parser.write("  \x1b[33m警告：\x1b[0m这是一条中文提示信息\r\n".as_bytes());
    let _ = parser.write("  \x1b[32m成功：\x1b[0m操作已完成，终端截图生成。\r\n".as_bytes());
    parser.screen().clone()
}

fn main() {
    let screen = build_screen();

    // Warm up
    for _ in 0..3 {
        let config = vibetty_screenshot::ScreenshotConfig {
            font_size: 16.0,
            padding: 24,
            background_color: [30, 30, 30, 255],
            show_decorations: true,
            title: Some("bench".to_string()),
        };
        let _ = vibetty_screenshot::capture_screen(&screen, &config);
    }

    // Benchmark
    let iterations = 50;
    let start = Instant::now();
    for _ in 0..iterations {
        let config = vibetty_screenshot::ScreenshotConfig {
            font_size: 16.0,
            padding: 24,
            background_color: [30, 30, 30, 255],
            show_decorations: true,
            title: Some("bench".to_string()),
        };
        let _ = vibetty_screenshot::capture_screen(&screen, &config);
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed / iterations;

    println!(
        "{} iterations: {:.2?} total, {:.2?}/frame",
        iterations, elapsed, per_frame
    );
}
