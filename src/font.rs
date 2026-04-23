//! Font loading utilities
//!
//! Delegates to the appropriate backend based on the active feature flag.

#[cfg(feature = "freetype")]
#[path = "font_freetype.rs"]
mod font_impl;

#[cfg(feature = "ab_glyph")]
#[path = "font_abglyph.rs"]
mod font_impl;

pub use font_impl::*;
