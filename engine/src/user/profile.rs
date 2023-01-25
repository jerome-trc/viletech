//! A named collection of information of which the user can keep multiple.

use crate::gfx::Rgb32;

/// A named collection of information of which the user can keep multiple.
///
/// Encompasses a p-storage sub-file system, aesthetic choices, saved games,
/// screenshots, and demos, but not preferences; those are kept in separate
/// "presets" so the user can mix and match them.
#[derive(Debug)]
pub struct Profile {
	/// Must be between 2 and 64 characters long, but is otherwise unrestricted.
	name: String,
	gender: Gender,
	/// Applied to the player's sprites.
	tint: Rgb32,
}

impl Profile {
	#[must_use]
	pub(super) fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub(super) fn new(name: String) -> Self {
		Self {
			name,
			gender: Gender::Neutral,
			tint: Rgb32::new(50, 200, 0),
		}
	}
}

impl Default for Profile {
	fn default() -> Self {
		Self {
			name: String::default(),
			gender: Gender::Neutral,
			tint: Rgb32::default(),
		}
	}
}

/// Used for pronouns during string formatting, such as in obituaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
	/// "she", "her", "her".
	Female,
	/// "he", "him", "his".
	Male,
	/// "they", "them", "their".
	Neutral,
	/// "it", "its".
	Object,
}
