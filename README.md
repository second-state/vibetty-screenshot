# vibetty-screenshot

Render vt100 terminal screens to PNG images.

## Usage

```rust
use std::io::Write;
use vibetty_screenshot::{save_screen_png, ScreenshotConfig};

// Create a 24x80 terminal and feed some output
let mut parser = vt100::Parser::new(24, 80, 0);
parser.write(b"\x1b[1;32mHello\x1b[0m, world!\r\n").unwrap();

let screen = parser.screen().clone();

// Configure and save
let config = ScreenshotConfig {
    font_size: 16.0,
    padding: 24,
    background_color: [30, 30, 30, 255],
    show_decorations: true,
    title: Some("Terminal".to_string()),
};

save_screen_png(&screen, "output.png", &config).unwrap();
```

## Example

```bash
cargo run --example basic
```

## Dependencies

- `vt100` — terminal screen parser
- `image` / `imageproc` — image rendering
- `ab_glyph` — font rendering
- `tiny-skia` — canvas background
