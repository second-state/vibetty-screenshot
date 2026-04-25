//! Font loading utilities
//!
//! Delegates to the appropriate backend based on the active feature flag.
//! Priority: swash > freetype > ab_glyph (default)

#[cfg(all(feature = "freetype", not(feature = "swash")))]
#[path = "font_freetype.rs"]
mod font_impl;

#[cfg(all(feature = "ab_glyph", not(feature = "swash"), not(feature = "freetype")))]
#[path = "font_abglyph.rs"]
mod font_impl;

#[cfg(feature = "swash")]
#[path = "font_swash.rs"]
mod font_impl;

pub use font_impl::*;
