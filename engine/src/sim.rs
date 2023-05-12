//! Home of all gameplay code.

pub mod actor;
pub mod level;
pub mod line;
pub mod sector;
pub mod setup;
pub mod skill;

use std::time::{Duration, Instant};

use bevy::{pbr::wireframe::Wireframe, prelude::*};
use nanorand::WyRand;

use crate::{
	data::dobj::{self},
	rng::RngCore,
};

/// All gameplay simulation state.
#[derive(Resource, Debug)]
pub struct Sim {
	timing: Timing,
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

/// A "flag" component for marking entities as being part of an active level.
///
/// Level geometry and actors without this are not subject to per-tick iteration.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct ActiveMarker;

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

pub fn start(mut cmds: Commands, context: setup::Context, level: dobj::Handle<dobj::Level>) {
	let start_time = Instant::now();

	let l = level.clone();

	cmds.spawn((
		GlobalTransform::default(),
		ComputedVisibility::default(),
		Wireframe,
		ActiveMarker,
	))
	.with_children(|cbuilder| {
		for thingdef in &level.things {
			if thingdef.num == 1 {
				cbuilder.spawn(Camera3dBundle {
					transform: Transform::from_xyz(thingdef.pos.x, 0.001, thingdef.pos.z),
					..default()
				});

				break;
			}
		}

		setup::level::setup(context, level, cbuilder);
	});

	info!(
		"Sim setup complete ({}) in {}ms.",
		&l.id(),
		start_time.elapsed().as_millis()
	);
}
