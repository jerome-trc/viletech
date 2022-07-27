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

use std::{collections::HashMap, fmt};

use nanorand::{WyRand, Rng};

#[derive(Debug)]
pub enum Error {
	KeyOverlap
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
			Self::KeyOverlap => {
				write!(f, "Tried to insert an RNG under an already-registered key.")
			}
		}
    }
}

pub trait ImpureRng {
	fn range_i32(&mut self, min_incl: i32, max_incl: i32) -> i32;
	fn range_f32(&mut self, min_incl: f32, max_incl: f32) -> f32;
	fn coin_flip(&mut self) -> bool;
}

impl ImpureRng for WyRand {
	fn range_i32(&mut self, min_incl: i32, max_incl: i32) -> i32 {
		self.generate_range(min_incl..(max_incl + 1))
	}

	fn range_f32(&mut self, min_incl: f32, max_incl: f32) -> f32 {
		min_incl + (self.generate::<f32>() / (1.0 / (max_incl - min_incl)))
	}

	fn coin_flip(&mut self) -> bool {
		self.generate()
	}
}

/// Contains maps of named random number generators.
pub struct RngCore<B: ImpureRng + Default> {
	prngs: HashMap<String, B>
}

impl<B: ImpureRng + Default> Default for RngCore<B> {
	fn default() -> Self {
		let mut ret = RngCore {
			prngs: Default::default()
		};

		ret.prngs.insert("".to_string(), B::default());

		ret
	}
}

impl<B: ImpureRng + Default> RngCore<B> {
	pub fn add_default(&mut self, key: String) -> Result<(), Error> {
		if self.prngs.contains_key(&key) {
			return Err(Error::KeyOverlap);
		}

		self.prngs.insert(key, B::default());

		Ok(())
	}

	pub fn add(&mut self, key: String, prng: B) -> Result<(), Error> {
		if self.prngs.contains_key(&key) {
			return Err(Error::KeyOverlap);
		}

		self.prngs.insert(key, prng);

		Ok(())
	}

	pub fn range_i32(&mut self, min_incl: i32, max_incl: i32) -> i32 {
		self.prngs.get_mut("").unwrap().range_i32(min_incl, max_incl)
	}

	pub fn range_f32(&mut self, min_incl: f32, max_incl: f32) -> f32 {
		self.prngs.get_mut("").unwrap().range_f32(min_incl, max_incl)
	}

	pub fn coinflip(&mut self) -> bool {
		self.prngs.get_mut("").unwrap().coin_flip()
	}
}
