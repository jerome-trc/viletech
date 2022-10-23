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
	error::Error,
	io,
	ops::{Deref, DerefMut},
	time::Duration,
};

use kira::{
	manager::{error::PlaySoundError, AudioManager},
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundHandle, StaticSoundSettings},
		SoundData,
	},
	tween::Tween,
};
use shipyard::EntityId;

use crate::VfsHandle;

pub struct SourcedHandle {
	inner: StaticSoundHandle,
	#[allow(unused)]
	source: Option<EntityId>,
}

impl Deref for SourcedHandle {
	type Target = StaticSoundHandle;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for SourcedHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

pub struct AudioCore {
	pub manager: AudioManager,
	/// General-purpose music slot.
	pub music1: Option<StaticSoundHandle>,
	/// Secondary music slot. Allows scripts to set a song to pause the level's
	/// main song, briefly play another piece, and then carry on with `music1`
	/// wherever it left off.
	pub music2: Option<StaticSoundHandle>,
	/// Sounds currently being played.
	pub sounds: Vec<SourcedHandle>,
}

pub type PlayError = PlaySoundError<<StaticSoundData as SoundData>::Error>;

impl AudioCore {
	/// Clear handles for sounds which have finished playing.
	pub fn update(&mut self) {
		let mut i = 0;

		while i < self.sounds.len() {
			if self.sounds[i].state() == PlaybackState::Stopped {
				self.sounds.swap_remove(i);
			} else {
				i += 1;
			}
		}
	}

	/// Play a sound without an in-world source.
	/// Always audible to all clients, not subject to panning or attenuation.
	pub fn play_global(&mut self, data: StaticSoundData) -> Result<(), PlayError> {
		self.sounds.push(SourcedHandle {
			inner: self.manager.play(data)?,
			source: None,
		});

		Ok(())
	}

	pub fn play_sourced(
		&mut self,
		data: StaticSoundData,
		entity: EntityId,
	) -> Result<(), PlayError> {
		self.sounds.push(SourcedHandle {
			inner: self.manager.play(data)?,
			source: Some(entity),
		});

		Ok(())
	}

	pub fn pause_all(&mut self) {
		let tween = tween_instant();

		for handle in &mut self.sounds {
			let res = handle.pause(tween);
			debug_assert!(res.is_ok(), "Failed to pause a sound: {}", res.unwrap_err());
		}

		if let Some(mus) = &mut self.music1 {
			let res = mus.pause(tween);
			debug_assert!(res.is_ok(), "Failed to pause music 1: {}", res.unwrap_err());
		}

		if let Some(mus) = &mut self.music2 {
			let res = mus.pause(tween);
			debug_assert!(res.is_ok(), "Failed to pause music 2: {}", res.unwrap_err());
		}
	}

	pub fn resume_all(&mut self) {
		let tween = tween_instant();

		for handle in &mut self.sounds {
			let res = handle.resume(tween);
			debug_assert!(
				res.is_ok(),
				"Failed to resume a sound: {}",
				res.unwrap_err()
			);
		}

		if let Some(mus) = &mut self.music1 {
			let res = mus.resume(tween);
			debug_assert!(
				res.is_ok(),
				"Failed to resume music 1: {}",
				res.unwrap_err()
			);
		}

		if let Some(mus) = &mut self.music2 {
			let res = mus.resume(tween);
			debug_assert!(
				res.is_ok(),
				"Failed to resume music 2: {}",
				res.unwrap_err()
			);
		}
	}
}

pub fn sound_from_file(
	handle: VfsHandle,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Box<dyn Error>> {
	let bytes = handle.read()?.to_owned();
	let cursor = io::Cursor::new(bytes);

	match StaticSoundData::from_cursor(cursor, settings) {
		Ok(ssd) => Ok(ssd),
		Err(err) => Err(Box::new(err)),
	}
}

pub fn tween_instant() -> Tween {
	Tween {
		start_time: kira::StartTime::Immediate,
		duration: Duration::default(),
		easing: kira::tween::Easing::Linear,
	}
}
