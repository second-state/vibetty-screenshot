use std::io::Write;

use vibetty_screenshot::{save_screen_png, ScreenshotConfig};

fn main() {
    // Create a 24x80 terminal
    let mut parser = vt100::Parser::new(24, 80, 0);

    // Feed some terminal output with ANSI colors
    parser.write(b"\x1b[1;32m  vibetty-screenshot\x1b[0m\r\n\r\n");
    parser.write(b"  Hello from the terminal!\r\n\r\n");
    parser.write(b"  \x1b[33mWarning:\x1b[0m this is an example\r\n");
    parser.write(b"  \x1b[31mError:\x1b[0m   something went wrong\r\n");
    parser.write(b"  \x1b[34mInfo:\x1b[0m    everything is fine\r\n\r\n");
    parser.write(b"  Rendering \x1b[1mbold\x1b[0m and \x1b[2mdim\x1b[0m text.\r\n");

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
