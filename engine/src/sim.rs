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

use crate::{data::game::DataCore, rng::RngCore};

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
	SpeedUp,
	SlowDown,
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

const WAIT_TIMES: [u64; 21] = [
	28_571 * 11, // -10
	28_571 * 10,
	28_571 * 9,
	28_571 * 8,
	28_571 * 7,
	28_571 * 6,
	28_571 * 5,
	28_571 * 4,
	28_571 * 3,
	28_571 * 2,
	28_571, // 0: normal, 35 tics/second
	28_571 / 2,
	28_571 / 3,
	28_571 / 4,
	28_571 / 5,
	28_571 / 6,
	28_571 / 7,
	28_571 / 8,
	28_571 / 9,
	28_571 / 10,
	28_571 / 11, // +10
];

const BASE_SIM_SPEED_INDEX: usize = 10;

pub fn run<C: EgressConfig>(context: Context) {
	let Context {
		lua,
		data: _,
		receiver,
		sender,
	} = context;

	// Ensure channels are unbounded
	debug_assert!(receiver.capacity().is_none());
	debug_assert!(sender.capacity().is_none());

	let mut speed_index = BASE_SIM_SPEED_INDEX;

	'sim: loop {
		let now = Instant::now();
		let next_tic = now + Duration::from_micros(WAIT_TIMES[speed_index]);
		let lua = lua.lock();
		let playsim = lua.app_data_mut::<PlaySim>().unwrap();

		while let Ok(msg) = receiver.try_recv() {
			match msg {
				InMessage::Stop => {
					break 'sim;
				}
				InMessage::SpeedUp => {
					speed_index = WAIT_TIMES.len().min(speed_index + 1);
				}
				InMessage::SlowDown => {
					speed_index = 0.max(speed_index - 1);
				}
			}
		}

		// ???

		drop(playsim);
		drop(lua);

		// If it took longer than the expected interval to process this tic,
		// increase the time dilation
		if Instant::now() > next_tic {
			speed_index -= 1;
		}

		std::thread::sleep(next_tic - now);
	}
}
