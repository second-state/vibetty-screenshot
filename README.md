# vibetty-screenshot

Render vt100 terminal screens to images.

Takes a `vt100::Screen` and produces an `image::RgbaImage` with proper ANSI color support, CJK (wide) character rendering, per-character font fallback, and optional window title bar decoration.

## Usage

```rust
use std::io::Write;
use vibetty_screenshot::{save_screen_png, ScreenshotConfig, Theme};

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
    theme: Theme::default(),
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

## Fonts

Two fonts are embedded and combined via per-character fallback:

- **JetBrains Mono** (primary, [OFL-1.1](assets/LICENSE-JetBrainsMono.txt)) — a monospace font for Latin/ASCII and box-drawing characters.
- **Sarasa Mono SC** (fallback, OFL-1.1) — renders CJK and any glyph the primary font lacks (JetBrains Mono contains no CJK).

Each character is looked up in the primary font first; if it has no glyph there, the fallback font renders it. This keeps Latin text crisp and properly monospaced while still supporting Chinese and other scripts.

## Feature Flags

Three font rendering backends are available via Cargo features. `ab_glyph` is the default (pure Rust, no C dependencies):

| Feature | Backend | Notes |
|---------|---------|-------|
| `ab_glyph` (default) | ab_glyph + imageproc (pure Rust) | No C dependencies |
| `freetype` | FreeType (statically compiled) | High-quality hinted rendering |
| `swash` | swash (pure Rust) | Pure Rust with built-in cache |

Only one backend should be enabled at a time; when several are set, priority is `swash` > `freetype` > `ab_glyph`.

```toml
# Default: ab_glyph (pure Rust)
vibetty-screenshot = "0.2"

# FreeType backend
vibetty-screenshot = { version = "0.2", default-features = false, features = ["freetype"] }

# swash backend
vibetty-screenshot = { version = "0.2", default-features = false, features = ["swash"] }
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
- `ab_glyph` + `imageproc` — font rendering (optional, default)
- `freetype-rs` — font rendering (optional)
- `swash` — font rendering (optional)
