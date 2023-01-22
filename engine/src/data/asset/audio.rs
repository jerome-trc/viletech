//! Sounds and music assets.

use kira::sound::static_sound::StaticSoundData;

use crate::audio::MidiData;

use super::Asset;

#[derive(Debug)]
pub struct Music {
	pub audio: Audio,
	// Q: Metadata?
}

#[derive(Debug)]
pub struct Sound {
	pub audio: Audio,
}

/// Storage for [`Music`] and [`Sound`], which have different needs of their own.
#[derive(Debug)]
pub enum Audio {
	Midi(MidiData),
	Waveform(StaticSoundData),
}

impl Asset for Music {}
impl Asset for Sound {}
