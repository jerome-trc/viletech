//! Graphics symbols specific to Doom games, such as PLAYPAL and ENDOOM.

pub struct ColorMap(pub [u8; 256]);

pub struct Endoom {
	colors: [u8; 2000],
	text: [u8; 2000],
}

impl Endoom {
	pub fn new(lump: &[u8]) -> Self {
		let mut ret = Self {
			colors: [0; 2000],
			text: [0; 2000],
		};

		let mut i = 0;

		while i < 4000 {
			ret.colors[i] = lump[i];
			ret.text[i] = lump[i + 1];
			i += 2;
		}

		ret
	}

	pub fn is_blinking(&self, index: usize) -> bool {
		debug_assert!(index < 2000);
		self.colors[index] & (1 << 7) == (1 << 7)
	}
}
