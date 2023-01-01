//! Home of all gameplay code.

use std::{
	sync::Arc,
	thread::JoinHandle,
	time::{Duration, Instant},
};

use nanorand::WyRand;
use parking_lot::RwLock;

use crate::{
	data::DataCore,
	ecs::{Components, DenseRegistry},
	rng::RngCore,
};

pub struct PlaySim {
	pub rng: RngCore<WyRand>,
	pub entities: DenseRegistry,
	pub components: Components,
}

impl Default for PlaySim {
	/// This constructor exists for easy testing/mocking/placeholder code but
	/// is not intended for use in any final implementations.
	fn default() -> Self {
		Self {
			rng: Default::default(),
			entities: DenseRegistry::new(521),
			components: Components::new(521),
		}
	}
}

pub struct Handle {
	pub sim: Arc<RwLock<PlaySim>>,
	pub sender: InSender,
	pub receiver: OutReceiver,
	pub thread: JoinHandle<()>,
}

pub enum InMessage {
	Stop,
	IncreaseTicRate,
	DecreaseTicRate,
	SetTicRate(i8),
}

/// Outbound messages are only for playsims running within the client.
pub enum OutMessage {
	Toast(String),
}

pub type InSender = crossbeam::channel::Sender<InMessage>;
pub type InReceiver = crossbeam::channel::Receiver<InMessage>;
pub type OutSender = crossbeam::channel::Sender<OutMessage>;
pub type OutReceiver = crossbeam::channel::Receiver<OutMessage>;

pub struct Context {
	pub sim: Arc<RwLock<PlaySim>>,
	pub data: Arc<RwLock<DataCore>>,
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
		data: _,
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
		let sim = sim.write();

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

		// ???

		drop(sim);

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
