//! "Extras" are items of game data that are not [`Datum`] but datum-adjacent.
//!
//! [`Datum`]: super::Datum

use image::Rgba;

/// See <https://doomwiki.org/wiki/COLORMAP>.
#[derive(Debug, Clone)]
pub struct ColorMap(pub [[u8; 256]; 34]);

impl ColorMap {
	/// A sensible default for internal use. All bytes are zero.
	#[must_use]
	pub(in super::super) fn black() -> Self {
		Self(unsafe { std::mem::zeroed() })
	}
}

/// See <https://doomwiki.org/wiki/ENDOOM>.
#[derive(Debug, Clone)]
pub struct EnDoom {
	pub colors: [u8; 2000],
	pub text: [u8; 2000],
}

impl EnDoom {
	#[must_use]
	pub fn is_blinking(&self, index: usize) -> bool {
		debug_assert!(index < 2000);
		self.colors[index] & (1 << 7) == (1 << 7)
	}
}

#[derive(Debug)]
pub struct Palette(pub [Rgba<f32>; 256]);

impl Palette {
	/// A sensible default for internal use. All colors are `0.0 0.0 0.0 1.0`.
	#[must_use]
	pub(in super::super) fn black() -> Self {
		Self([Rgba([0.0, 0.0, 0.0, 1.0]); 256])
	}
}

#[derive(Debug)]
pub struct PaletteSet(pub [Palette; 14]);
