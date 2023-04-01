//! Sounds and music assets.

use kira::sound::static_sound::StaticSoundData;

use crate::audio::MidiData;

use super::{Asset, AssetKind, Record};

/// Storage for [`Music`] and [`Sound`], which have different needs of their own.
#[derive(Debug)]
pub enum Audio {
	Midi(MidiData),
	Waveform(StaticSoundData),
}

impl Asset for Audio {
	const KIND: AssetKind = AssetKind::Audio;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.audio
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.audio
	}
}
