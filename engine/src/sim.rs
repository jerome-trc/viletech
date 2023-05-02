//! Home of all gameplay code.

pub mod ecs;
pub mod level;

use std::time::{Duration, Instant};

use bevy::prelude::*;
use nanorand::WyRand;

use crate::rng::RngCore;

use self::level::Level;

/// All gameplay simulation state.
#[derive(Debug, Resource)]
pub struct Sim {
	timing: Timing,
	_levels: Vec<Level>,
	_rng: RngCore<WyRand>,
	/// Time spent in this hub thus far.
	hub_ticks_elapsed: u64,
	/// Time spent in this playthrough thus far.
	ticks_elapsed: u64,
}

/// Separate from [`Sim`] for cleanliness.
#[derive(Debug)]
struct Timing {
	/// `true` by default. If enabled, the sim will slow down whenever a tick-rate
	/// of 35 Hz becomes unsustainable in order to prevent frame skipping.
	dilate: bool,
	/// 0 by default. Clamped to the range `-10..=10`.
	/// See [`Self::tick_interval`] to understand how this value factors in.
	tweak_real: i64,
	/// 0 by default. Clamped to the range `-10..=10`.
	/// The user sets this value to attempt to change the simulation rate.
	/// The scheduler will aim to keep `tweak_real` equivalent to this, but will
	/// set it lower if necessary to prevent the renderer from missing frames
	/// (note that this behavior never happens at all if `dilate` is `false`).
	tweak_goal: i64,
}

impl Timing {
	#[must_use]
	fn tick_interval(&self) -> Duration {
		let base = Duration::from_secs_f64(1.0 / 35.0);

		if self.tweak_real >= 0 {
			base + Duration::from_millis((self.tweak_real as u64) * 2)
		} else {
			base - Duration::from_millis((self.tweak_real.unsigned_abs()) * 2)
		}
	}
}

impl Default for Timing {
	fn default() -> Self {
		Self {
			dilate: true,
			tweak_real: 0,
			tweak_goal: 0,
		}
	}
}

/// Intended to be run on a fixed-time update loop, 35 Hz by default.
pub fn tick(mut sim: ResMut<Sim>, mut fixed_time: ResMut<FixedTime>) {
	let deadline = Instant::now() + sim.timing.tick_interval();

	sim.ticks_elapsed += 1;
	sim.hub_ticks_elapsed += 1;

	if sim.timing.dilate {
		if Instant::now() > deadline && sim.timing.tweak_real > -10 {
			sim.timing.tweak_real = (sim.timing.tweak_real - 1).max(-10);
		} else if sim.timing.tweak_real < sim.timing.tweak_goal {
			sim.timing.tweak_real = (sim.timing.tweak_real + 1).min(10);
		}

		fixed_time.period = sim.timing.tick_interval();
	}
}
