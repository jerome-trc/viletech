//! Graphics symbols specific to Doom games, such as PLAYPAL and ENDOOM.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use palette::rgb::Rgb;

use super::Rgb32;

pub struct Palette(pub [Rgb32; 256]);

impl Default for Palette {
	fn default() -> Self {
		Palette([Rgb::default(); 256])
	}
}

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
