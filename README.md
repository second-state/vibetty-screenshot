# vibetty-screenshot

Render vt100 terminal screens to images.

Takes a `vt100::Screen` and produces an `image::RgbaImage` with proper ANSI color support, CJK (wide) character rendering, and optional window title bar decoration.

## Usage

```rust
use std::io::Write;
use vibetty_screenshot::{save_screen_png, ScreenshotConfig};

// Create a terminal and feed some output
let mut parser = vt100::Parser::new(24, 80, 0);
parser.write(b"\x1b[1;32mHello\x1b[0m, world!\r\n").unwrap();

let screen = parser.screen().clone();

let config = ScreenshotConfig {
    font_size: 16.0,
    padding: 24,
    background_color: [30, 30, 30, 255],
    show_decorations: true,
    title: Some("Terminal".to_string()),
};

// Save directly to PNG
save_screen_png(&screen, "output.png", &config).unwrap();
```

To get the image object instead of writing a file:

```rust
use vibetty_screenshot::capture_screen;

let image = capture_screen(&screen, &config).unwrap();
// image is an image::RgbaImage — encode as JPEG, PNG, resize, etc.
```

## Feature Flags

Two font rendering backends are available via Cargo features:

| Feature | Backend | Notes |
|---------|---------|-------|
| `freetype` (default) | FreeType (statically compiled) | High-quality hinted rendering |
| `ab_glyph` | ab_glyph + imageproc (pure Rust) | No C dependencies |

```toml
# Default: FreeType
vibetty-screenshot = "0.1"

# Pure Rust backend
vibetty-screenshot = { version = "0.1", default-features = false, features = ["ab_glyph"] }
```

## Example

```bash
cargo run --example basic
```

## API

```rust
pub fn capture_screen(screen: &vt100::Screen, config: &ScreenshotConfig) -> Result<RgbaImage, ScreenshotError>
pub fn save_screen_png(screen: &vt100::Screen, path: &str, config: &ScreenshotConfig) -> Result<(), ScreenshotError>
```

## Dependencies

- `vt100` — terminal screen parser
- `image` — image encoding/decoding
- `tiny-skia` — canvas shape rendering
- `freetype-rs` — font rendering (optional, default)
- `ab_glyph` + `imageproc` — font rendering (optional)
