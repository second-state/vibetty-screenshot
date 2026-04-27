use std::io::Write;

use vibetty_screenshot::{save_screen_png, ScreenshotConfig};

fn main() {
    // Create a 24x80 terminal
    let mut parser = vt100::Parser::new(24, 80, 0);

    // Feed some terminal output with ANSI colors
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

    // Block characters — should tile seamlessly
    let _ = parser.write(b"\r\n");
    let _ = parser.write("  \x1b[42m                                                                          \x1b[0m\r\n".as_bytes());
    let _ = parser.write(b"\r\n");
    let _ = parser.write("  Full: \x1b[44m████████\x1b[0m  Upper: \x1b[43m▀▀▀▀▀▀▀▀\x1b[0m\r\n".as_bytes());
    let _ = parser.write("  Lower: \x1b[41m▄▄▄▄▄▄▄▄\x1b[0m  Left: \x1b[45m▌▌▌▌▌▌▌▌\x1b[0m\r\n".as_bytes());
    let _ = parser.write("  Shade: \x1b[46m░░▒▒▓▓██\x1b[0m\r\n".as_bytes());
    let _ = parser.write("  Progress: \x1b[42m████████  \x1b[0m\x1b[43m▀▀▀▀\x1b[0m\r\n".as_bytes());

    let screen = parser.screen().clone();

    let config = ScreenshotConfig {
        font_size: 16.0,
        padding: 24,
        background_color: [30, 30, 30, 255],
        show_decorations: true,
        title: Some("vibetty-screenshot example".to_string()),
    };

    save_screen_png(&screen, "output.png", &config).expect("Failed to save screenshot");
    println!("Screenshot saved to output.png");
}
