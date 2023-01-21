//! Home of all gameplay code.

mod ecs;
pub mod level;

use std::{
	sync::Arc,
	thread::JoinHandle,
	time::{Duration, Instant},
};

use nanorand::WyRand;
use parking_lot::RwLock;

use crate::{data::Catalog, player::Player, rng::RngCore};

pub use self::ecs::*;
use self::level::Level;

#[derive(Debug)]
pub struct PlaySim {
	pub players: Vec<Player>,
	pub rng: RngCore<WyRand>,
	pub level: Level,
	pub ecs: World,
	/// Time spent in this hub thus far.
	pub hub_tics_elapsed: u64,
	/// Time spent in this playthrough thus far.
	pub tics_elapsed: u64,
}

impl Default for PlaySim {
	fn default() -> Self {
		Self {
			players: Vec::default(),
			rng: RngCore::default(),
			level: Level::default(),
			ecs: World::new(521),
			hub_tics_elapsed: 0,
			tics_elapsed: 0,
		}
	}
}

#[derive(Debug)]
pub struct Handle {
	pub sim: Arc<RwLock<PlaySim>>,
	pub sender: InSender,
	pub receiver: OutReceiver,
	pub thread: JoinHandle<()>,
}

#[derive(Debug)]
pub enum InMessage {
	Stop,
	IncreaseTicRate,
	DecreaseTicRate,
	SetTicRate(i8),
}

/// Outbound messages are only for playsims running within the client.
#[derive(Debug)]
pub enum OutMessage {
	Toast(String),
}

pub type InSender = crossbeam::channel::Sender<InMessage>;
pub type InReceiver = crossbeam::channel::Receiver<InMessage>;
pub type OutSender = crossbeam::channel::Sender<OutMessage>;
pub type OutReceiver = crossbeam::channel::Receiver<OutMessage>;

#[derive(Debug)]
pub struct Context {
	pub sim: Arc<RwLock<PlaySim>>,
	pub catalog: Arc<RwLock<Catalog>>,
	pub receiver: InReceiver,
	pub sender: OutSender,
}

bitflags::bitflags! {
	pub struct Config: u8 {
		/// If not set, the sim thread will not send out client messages
		/// (e.g. sound, particles, screen effects).
		const CLIENT = 1 << 0;
	}
}

/// Pass [`Config::bits`] to `CFG`.
///
/// Note that the idea here is explicitly to generate versions of this function
/// and others it calls for every possible sim configuration to eliminate branches.
/// This may well generate an oversized binary and/or bloat the instruction cache,
/// negating performance gains. Benchmarks will eventually be needed.
pub fn run<const CFG: u8>(context: Context) {
	const BASE_TICINTERVAL: u64 = 28_571; // In microseconds
	const BASE_TICINTERVAL_INDEX: usize = 10;

	#[rustfmt::skip]
	const TICINTERVAL_POWERS: [f64; 21] = [
		// Minimum speed (-10 in the UI) is approximately 12 tics per second
		1.10, 1.09, 1.08, 1.07, 1.06,
		1.05, 1.04, 1.03, 1.02, 1.01,
		1.00, // Base speed (0 in the UI) is 35 tics per second
		0.99, 0.98, 0.97, 0.96, 0.95,
		0.94, 0.93, 0.92, 0.91, 0.90,
		// Maximum speed (+10 in the UI) is approximately 97 tics per second
	];

	fn calc_tic_interval(index: usize) -> u64 {
		(BASE_TICINTERVAL as f64)
			.powf(TICINTERVAL_POWERS[index])
			.round() as u64
	}

	let Context {
		sim,
		catalog: _,
		receiver,
		sender,
	} = context;

	// Ensure channels are unbounded
	debug_assert!(receiver.capacity().is_none());
	debug_assert!(sender.capacity().is_none());

	let mut tindx_real = BASE_TICINTERVAL_INDEX;
	let mut tindx_goal = BASE_TICINTERVAL_INDEX;
	let mut tic_interval = BASE_TICINTERVAL; // In microseconds

	'sim: loop {
		let now = Instant::now();
		let next_tic = now + Duration::from_micros(tic_interval);

		while let Ok(msg) = receiver.try_recv() {
			match msg {
				InMessage::Stop => {
					break 'sim;
				}
				InMessage::IncreaseTicRate => {
					if tindx_goal < (TICINTERVAL_POWERS.len() - 1) {
						tindx_goal += 1;
					}
				}
				InMessage::DecreaseTicRate => {
					tindx_goal = tindx_goal.saturating_sub(1);
				}
				InMessage::SetTicRate(s_ticrate) => {
					debug_assert!((-10..=10).contains(&s_ticrate));
					tindx_goal = (s_ticrate + 10) as usize;
				}
			}
		}

		unsafe {
			tick::<CFG>(sim.write(), &receiver, &sender);
		}

		// If it took longer than the expected interval to process this tic,
		// increase the time dilation; if it took less, try to go back up to
		// the user's desired tic rate
		if Instant::now() > next_tic && tindx_real > 0 {
			tindx_real -= 1;
			tic_interval = calc_tic_interval(tindx_real);
		} else if tindx_real < tindx_goal {
			tindx_real += 1;
			tic_interval = calc_tic_interval(tindx_real);
		}

		std::thread::sleep(next_tic - now);
	}
}

type WriteGuard<'a> = parking_lot::RwLockWriteGuard<'a, PlaySim>;

/// The critical section for sim tic execution. By taking a write guard, it is
/// guaranteed that any mutable references created to the playsim data are truly
/// exclusive.
///
/// In combination with the [`UnsafeCell`](std::cell::UnsafeCell) in parking_lot's
/// [`RwLock`], this function can be trusted to be the sole source of truth
/// for manipulating sim state, whether that be by itself or in Lith functions.
#[inline]
unsafe fn tick<const CFG: u8>(mut sim: WriteGuard, _receiver: &InReceiver, _sender: &OutSender) {
	'actors: for (core, _, _) in sim.ecs.comps.iter_mut_all() {
		if core.freeze_tics > 0 {
			core.freeze_tics -= 1;
			continue 'actors;
		}
	}

	sim.tics_elapsed += 1;
	sim.level.tics_elapsed += 1;
	sim.hub_tics_elapsed += 1;
}
