//! Home of all gameplay code.

pub mod ecs;
pub mod level;

use std::{
	sync::Arc,
	thread::JoinHandle,
	time::{Duration, Instant},
};

use nanorand::WyRand;
use parking_lot::RwLock;

use crate::{data::Catalog, lith, player::Player, rng::RngCore};

use self::level::Level;

pub use ecs::ActorId;

#[derive(Debug)]
pub struct PlaySim {
	pub(self) context: Context,
	pub state: RwLock<State>,
}

#[derive(Debug)]
struct Context {
	pub(self) catalog: Arc<RwLock<Catalog>>,
	pub(self) runtime: Arc<RwLock<lith::Runtime>>,
	pub(self) sender: OutSender,
	pub(self) receiver: InReceiver,
}

#[derive(Debug, Default)]
pub struct State {
	pub players: Vec<Player>, // Q: Null player 0, for non-zero optimizations?
	pub rng: RngCore<WyRand>,
	pub level: Level,
	/// Time spent in this hub thus far.
	pub hub_tics_elapsed: u64,
	/// Time spent in this playthrough thus far.
	pub tics_elapsed: u64,
}

impl PlaySim {
	#[must_use]
	pub fn new(
		catalog: Arc<RwLock<Catalog>>,
		runtime: Arc<RwLock<lith::Runtime>>,
		sender: OutSender,
		receiver: InReceiver,
	) -> Self {
		Self {
			context: Context {
				catalog,
				runtime,
				sender,
				receiver,
			},
			state: RwLock::new(State::default()),
		}
	}

	/// For the client to read all render state.
	/// - Sim state gets read-locked.
	/// - Lith runtime gets read-locked.
	/// - Catalog gets read-locked.
	#[must_use]
	pub fn read(&self) -> Ref {
		Ref {
			ctx: &self.context,
			catalog: self.context.catalog.read(),
			runtime: self.context.runtime.read(),
			state: self.state.read(),
		}
	}

	/// For the sim thread to run a tic and the client to unmark dirty level geometry.
	/// - Sim state gets write-locked.
	/// - Lith runtime gets write-locked.
	/// - Catalog gets read-locked.
	#[must_use]
	pub fn write(&self) -> RefMut {
		RefMut {
			ctx: &self.context,
			catalog: self.context.catalog.read(),
			runtime: self.context.runtime.write(),
			state: self.state.write(),
		}
	}
}

pub type WriteGuard<'s, T> = parking_lot::RwLockWriteGuard<'s, T>;
pub type ReadGuard<'s, T> = parking_lot::RwLockReadGuard<'s, T>;

/// See [`PlaySim::read`].
#[derive(Debug)]
pub struct Ref<'s> {
	pub(self) ctx: &'s Context,
	pub(self) catalog: ReadGuard<'s, Catalog>,
	pub(self) runtime: ReadGuard<'s, lith::Runtime>,
	pub(self) state: ReadGuard<'s, State>,
}

impl<'s> std::ops::Deref for Ref<'s> {
	type Target = ReadGuard<'s, State>;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

/// See [`PlaySim::write`].
#[derive(Debug)]
pub struct RefMut<'s> {
	pub(self) ctx: &'s Context,
	pub(self) catalog: ReadGuard<'s, Catalog>,
	pub(self) runtime: WriteGuard<'s, lith::Runtime>,
	pub(self) state: WriteGuard<'s, State>,
}

impl<'s> std::ops::Deref for RefMut<'s> {
	type Target = WriteGuard<'s, State>;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

impl<'s> std::ops::DerefMut for RefMut<'s> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.state
	}
}

/// Allows the main thread to interface with the sim thread.
#[derive(Debug)]
pub struct Handle {
	pub sim: Arc<PlaySim>,
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

pub fn run(sim: Arc<PlaySim>) {
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

	let mut tindx_real = BASE_TICINTERVAL_INDEX;
	let mut tindx_goal = BASE_TICINTERVAL_INDEX;
	let mut tic_interval = BASE_TICINTERVAL; // In microseconds

	'sim: loop {
		let tic_start = Instant::now();
		let next_tic_start = tic_start + Duration::from_micros(tic_interval);

		while let Ok(msg) = sim.context.receiver.try_recv() {
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

		tick(sim.write());

		// If it took longer than the expected interval to process this tic,
		// increase the time dilation; if it took less, try to go back up to
		// the user's desired tic rate
		if Instant::now() > next_tic_start && tindx_real > 0 {
			tindx_real -= 1;
			tic_interval = calc_tic_interval(tindx_real);
			continue 'sim; // Already behind schedule; don't sleep
		} else if tindx_real < tindx_goal {
			tindx_real += 1;
			tic_interval = calc_tic_interval(tindx_real);
		}

		// Q:
		// - Better precision with spin-locks for tail end of sleep interval?
		// - Better precision via exponential back-off?
		std::thread::sleep(next_tic_start - tic_start);
	}
}

/// The critical section for sim tic execution. By taking a write guard, it is
/// guaranteed that any mutable references created to the playsim data are truly
/// exclusive.
///
/// In combination with the [`UnsafeCell`](std::cell::UnsafeCell) in parking_lot's
/// [`RwLock`], this function can be trusted to be the sole source of truth
/// for manipulating sim state, whether that be by itself or in Lith functions.
#[inline]
fn tick(mut sim: RefMut) {
	sim.tics_elapsed += 1;
	sim.level.tics_elapsed += 1;
	sim.hub_tics_elapsed += 1;
}
