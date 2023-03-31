//! Symbols relating to pseudo-random number generation.

use std::{collections::HashMap, fmt};

use bevy::prelude::Resource;
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
/// alternative implementations (e.g. Boom-like, Doom-like, SFMT) can
/// hypothetically be built and substituted.
pub trait Prng: Default {
	#[must_use]
	fn range_i64(&mut self, min_incl: i64, max_incl: i64) -> i64;
	#[must_use]
	fn range_f64(&mut self, min_incl: f64, max_incl: f64) -> f64;
	#[must_use]
	fn range_usize(&mut self, min_incl: usize, max_incl: usize) -> usize;
	#[must_use]
	fn coin_flip(&mut self) -> bool;

	/// Returns a random character in the range from 0x61 to 0x7A.
	#[must_use]
	fn ascii_lowercase(&mut self) -> char {
		char::from_u32(self.range_usize(97, 122) as u32).unwrap()
	}

	/// Returns a random character in the range from 0x41 to 0x5A.
	#[must_use]
	fn ascii_uppercase(&mut self) -> char {
		char::from_u32(self.range_usize(65, 90) as u32).unwrap()
	}

	/// Returns a random character in the range from 0x30 to 0x39.
	#[must_use]
	fn ascii_digit(&mut self) -> char {
		char::from_u32(self.range_usize(48, 57) as u32).unwrap()
	}

	/// Returns a random character in the range from 0x30 to 0x39.
	#[must_use]
	fn ascii_char(&mut self) -> char {
		char::from_u32(self.range_usize(33, 126) as u32).unwrap()
	}
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
#[derive(Debug, Resource)]
pub struct RngCore<B: Prng> {
	prngs: HashMap<String, B>,
}

impl<B: Prng> Default for RngCore<B> {
	fn default() -> Self {
		RngCore {
			prngs: HashMap::from([(String::default(), B::default())]),
		}
	}
}

impl<B: Prng> RngCore<B> {
	/// Panics if there is already a PRNG under `key`.
	/// Check before-hand with [`Self::contains`].
	pub fn add_default(&mut self, key: String) {
		assert!(
			!self.contains(&key),
			"Tried to overwrite PRNG with key: {key}"
		);

		self.prngs.insert(key, B::default());
	}

	/// Panics if there is already a PRNG under `key`.
	/// Check before-hand with [`Self::contains`].
	pub fn add(&mut self, key: String, prng: B) {
		assert!(
			!self.contains(&key),
			"Tried to overwrite PRNG with key: {key}"
		);

		self.prngs.insert(key, prng);
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
	pub fn get_default(&mut self) -> &mut B {
		self.prngs.get_mut("").unwrap()
	}

	/// Returns true if a PRNG is already stored under the given key.
	#[must_use]
	pub fn contains(&self, key: &str) -> bool {
		self.prngs.contains_key(key)
	}
}
