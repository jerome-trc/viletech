//! Symbols relating to pseudo-random number generation.

use std::collections::HashMap;

use bevy::prelude::Resource;
use nanorand::{Rng, SeedableRng, WyRand};

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
			"tried to overwrite PRNG with key: {key}"
		);

		self.prngs.insert(key, B::default());
	}

	/// Panics if there is already a PRNG under `key`.
	/// Check before-hand with [`Self::contains`].
	pub fn add(&mut self, key: String, prng: B) {
		assert!(
			!self.contains(&key),
			"tried to overwrite PRNG with key: {key}"
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

// PRNG trait //////////////////////////////////////////////////////////////////

/// The PRNG behavior used by the engine all passes through this trait, so that
/// alternative implementations (e.g. Boom-like, Doom-like, SFMT) can
/// hypothetically be built and substituted.
pub trait Prng: Default {
	fn seed(&mut self, seed: u64);

	#[must_use]
	fn range_i64(&mut self, min_incl: i64, max_incl: i64) -> i64;
	#[must_use]
	fn range_f64(&mut self, min_incl: f64, max_incl: f64) -> f64;
	#[must_use]
	fn range_usize(&mut self, min_incl: usize, max_incl: usize) -> usize;

	// Provided ////////////////////////////////////////////////////////////////

	#[must_use]
	fn boolean(&mut self) -> bool {
		self.range_i64(0, 1) == 0
	}

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

	fn shuffle<T>(&mut self, mut target: impl AsMut<[T]>) {
		let target = target.as_mut();
		let len = target.len();

		for i in 0..len {
			let r = self.range_usize(0, len);
			target.swap(i, r);
		}
	}
}

// WyRand //////////////////////////////////////////////////////////////////////

impl Prng for WyRand {
	fn seed(&mut self, seed: u64) {
		self.reseed(seed.to_le_bytes());
	}

	fn range_i64(&mut self, min_incl: i64, max_incl: i64) -> i64 {
		self.generate_range(min_incl..=max_incl)
	}

	fn range_f64(&mut self, min_incl: f64, max_incl: f64) -> f64 {
		min_incl + (self.generate::<f64>() / (1.0 / (max_incl - min_incl)))
	}

	fn range_usize(&mut self, min_incl: usize, max_incl: usize) -> usize {
		self.generate_range(min_incl..=max_incl)
	}

	fn boolean(&mut self) -> bool {
		self.generate()
	}
}
