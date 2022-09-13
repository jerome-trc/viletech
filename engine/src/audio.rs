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

use kira::{
	manager::AudioManager,
	sound::static_sound::{PlaybackState, StaticSoundHandle},
};

pub struct AudioCore {
	pub manager: AudioManager,
	/// General-purpose music slot.
	pub music1: Option<StaticSoundHandle>,
	/// Secondary music slot. Allows scripts to set a song to pause the level's
	/// main song, briefly play another piece, and then carry on with `music1`
	/// wherever it left off.
	pub music2: Option<StaticSoundHandle>,
	/// Sounds currently being played.
	pub handles: Vec<StaticSoundHandle>,
}

impl AudioCore {
	/// Clear handles for sounds which have finished playing.
	pub fn update(&mut self) {
		let mut i = 0;

		while i < self.handles.len() {
			if self.handles[i].state() == PlaybackState::Stopped {
				self.handles.swap_remove(i);
			} else {
				i += 1;
			}
		}
	}
}
