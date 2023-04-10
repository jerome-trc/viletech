//! Helper functions that belong nowhere else in particular.

pub mod env;
pub mod io;
#[macro_use]
pub mod macros;
pub mod path;
pub mod string;

/// Note that minutes and seconds are both remainders, not totals.
#[must_use]
pub fn duration_to_hhmmss(duration: std::time::Duration) -> (u64, u64, u64) {
	let mins = duration.as_secs() / 60;
	let hours = mins / 60;
	(hours, mins % 60, duration.as_secs() % 60)
}
