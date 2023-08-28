//! "CMid" translates Lith ASTs to C for consumption by the [LithC] backend.
//!
//! [LithC]: crate::back::c

#[cfg(feature = "viletech")]
mod zscript;

#[derive(Debug)]
pub struct Source {
	pub name: String,
	pub text: String,
}
