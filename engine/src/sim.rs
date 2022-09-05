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
	sync::{atomic::AtomicBool, Arc},
	time::{Duration, Instant},
};

use nanorand::WyRand;
use parking_lot::RwLock;
use shipyard::World;

use crate::rng::RngCore;

#[derive(Default)]
pub struct PlaySim {
	rng: RngCore<WyRand>,
	world: World,
}

pub enum InMessage {
	// ???
}

pub struct ThreadContext {
	pub playsim: Arc<RwLock<PlaySim>>,
	pub receiver: crossbeam::channel::Receiver<InMessage>,
	pub running: Arc<AtomicBool>,
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

pub fn run(context: ThreadContext) {
	let ThreadContext {
		playsim,
		receiver,
		running,
	} = context;
	let mut speed_index = 10;

	loop {
		if !running.load(std::sync::atomic::Ordering::Relaxed) {
			break;
		}

		let now = Instant::now();
		let next_tic = now + Duration::from_micros(WAIT_TIMES[speed_index]);

		let playsim = playsim.write();

		while let Ok(msg) = receiver.try_recv() {
			match msg {
				// ???	
			}
		}

		// ???

		drop(playsim);

		// If it took longer than the expected interval to process this tic,
		// increase the time dilation
		if Instant::now() > next_tic {
			speed_index -= 1;
		}

		std::thread::sleep(next_tic - now);
	}
}
