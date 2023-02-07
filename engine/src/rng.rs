//! Symbols relating to pseudo-random number generation.

use std::{collections::HashMap, fmt};

use nanorand::{Rng, WyRand};

#[derive(Debug)]
pub enum Error {
	KeyOverlap,
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

/// The PRNG behavior used by the engine all passes through this trait, so that
/// alternative implementations (e.g. Boom-like, Doom-like, SFMT) can hypothetically
/// be built and substituted.
pub trait Prng: Default {
	#[must_use]
	fn range_i64(&mut self, min_incl: i64, max_incl: i64) -> i64;
	#[must_use]
	fn range_f64(&mut self, min_incl: f64, max_incl: f64) -> f64;
	#[must_use]
	fn range_usize(&mut self, min_incl: usize, max_incl: usize) -> usize;
	#[must_use]
	fn coin_flip(&mut self) -> bool;
}

impl Prng for WyRand {
	fn range_i64(&mut self, min_incl: i64, max_incl: i64) -> i64 {
		self.generate_range(min_incl..=max_incl)
	}

	fn range_f64(&mut self, min_incl: f64, max_incl: f64) -> f64 {
		min_incl + (self.generate::<f64>() / (1.0 / (max_incl - min_incl)))
	}

	fn range_usize(&mut self, min_incl: usize, max_incl: usize) -> usize {
		self.generate_range(min_incl..=max_incl)
	}

	fn coin_flip(&mut self) -> bool {
		self.generate()
	}
}

/// Contains a map of named random number generators.
#[derive(Debug)]
pub struct RngCore<B: Prng> {
	prngs: HashMap<String, B>,
}

impl<B: Prng> Default for RngCore<B> {
	fn default() -> Self {
		let mut ret = RngCore {
			prngs: Default::default(),
		};

		ret.prngs.insert("".to_string(), B::default());

		ret
	}
}

impl<B: Prng> RngCore<B> {
	/// Returns an error if there's already a PRNG under `key`.
	pub fn add_default(&mut self, key: String) -> Result<(), Error> {
		if self.prngs.contains_key(&key) {
			return Err(Error::KeyOverlap);
		}

		self.prngs.insert(key, B::default());

		Ok(())
	}

	/// Returns an error if there's already a PRNG under `key`.
	pub fn add(&mut self, key: String, prng: B) -> Result<(), Error> {
		if self.prngs.contains_key(&key) {
			return Err(Error::KeyOverlap);
		}

		self.prngs.insert(key, prng);

		Ok(())
	}

	pub fn try_get(&mut self, key: &str) -> Option<&mut B> {
		self.prngs.get_mut(key)
	}

	/// Shortcut for `try_get().unwrap()`, for PRNGs which are provably known to
	/// be registered by the engine. Panics if there's no PRNG under `key`.
	pub fn get(&mut self, key: &str) -> &mut B {
		self.prngs.get_mut(key).unwrap()
	}

	/// Retrieves the PRNG behind the key "", used as a sensible default.
	pub fn get_anon(&mut self) -> &mut B {
		self.prngs.get_mut("").unwrap()
	}
}
