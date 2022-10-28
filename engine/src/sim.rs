/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	sync::Arc,
	thread::JoinHandle,
	time::{Duration, Instant},
};

use mlua::prelude::*;
use nanorand::WyRand;
use parking_lot::{Mutex, RwLock};
use shipyard::World;

use crate::{data::DataCore, lua::ImpureLua, rng::RngCore};

#[derive(Default)]
pub struct PlaySim {
	pub rng: RngCore<WyRand>,
	pub world: World,
}

pub struct Handle {
	pub sender: InSender,
	pub receiver: OutReceiver,
	pub thread: JoinHandle<()>,
}

pub trait EgressConfig {
	fn egress(sender: OutSender, msg: OutMessage);
}

pub struct EgressConfigClient;
pub struct EgressConfigNoop;

impl EgressConfig for EgressConfigClient {
	fn egress(sender: OutSender, msg: OutMessage) {
		let res = sender.send(msg);
		debug_assert!(
			res.is_ok(),
			"Failed to send sim egress message: {}",
			res.unwrap_err()
		);
	}
}

impl EgressConfig for EgressConfigNoop {
	fn egress(_: OutSender, _: OutMessage) {}
}

pub enum InMessage {
	Stop,
	IncreaseTicRate,
	DecreaseTicRate,
	SetTicRate(i8),
}

/// Outbound messages are only for playsims running within the client.
pub enum OutMessage {
	// ???
}

pub type InSender = crossbeam::channel::Sender<InMessage>;
pub type InReceiver = crossbeam::channel::Receiver<InMessage>;
pub type OutSender = crossbeam::channel::Sender<OutMessage>;
pub type OutReceiver = crossbeam::channel::Receiver<OutMessage>;

pub struct Context {
	pub lua: Arc<Mutex<Lua>>,
	pub data: Arc<RwLock<DataCore>>,
	pub receiver: InReceiver,
	pub sender: OutSender,
}

pub fn run<C: EgressConfig>(context: Context) {
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
		lua,
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
		let lua = lua.lock();
		lua.set_clientside(false);
		let playsim = lua.app_data_mut::<PlaySim>().unwrap();

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
					if tindx_goal > 0 {
						tindx_goal -= 1;
					}
				}
				InMessage::SetTicRate(s_ticrate) => {
					debug_assert!((-10..=10).contains(&s_ticrate));
					tindx_goal = (s_ticrate + 10) as usize;
				}
			}
		}

		// ???

		drop(playsim);
		lua.set_clientside(true);
		drop(lua);

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
