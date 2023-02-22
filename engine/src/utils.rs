//! Helper functions that belong nowhere else in particular.

pub mod env;
pub mod io;
pub mod lang;
#[macro_use]
pub mod macros;
pub mod path;
pub mod string;

/// Note that minutes and seconds are both remainders, not totals.
#[must_use]
pub fn duration_to_hhmmss(duration: std::time::Duration) -> (i64, i64, i64) {
	let duration = chrono::Duration::from_std(duration).unwrap();
	let secs = duration.num_seconds();
	let mins = secs / 60;
	let hours = mins / 60;
	(hours, mins % 60, secs % 60)
}
