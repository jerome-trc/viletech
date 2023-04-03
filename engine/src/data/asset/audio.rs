//! Sounds and music assets.
//!
//! Code derived from [SLADE](https://slade.mancubus.net/) is used under the
//! GNU GPL v2.0. See <https://github.com/sirjuddington/SLADE/blob/master/LICENSE>.
//! A copy is attached in the `/legal` directory.

use byteorder::{ByteOrder, LittleEndian};
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

impl Audio {
	/// Adapted from SLADE's `DoomPCSpeakerDataFormat::isThisFormat`.
	#[must_use]
	pub(in super::super) fn is_pc_speaker_sound(bytes: &[u8]) -> bool {
		if bytes.len() < 4 {
			return false;
		}

		// (SLADE) The first two bytes must always be NUL.
		if bytes[0] > 0 || bytes[1] > 0 {
			return false;
		}

		// (SLADE) Next is the number of samples (LE uint16_t), and each sample
		// is a single byte, so the size can be checked easily.

		let sample_count = 4 + LittleEndian::read_u16(&bytes[2..4]);

		if bytes.len() == sample_count as usize {
			return true;
		}

		false
	}

	#[must_use]
	pub(in super::super) fn is_dmxmus(bytes: &[u8]) -> bool {
		if bytes.len() < 4 {
			return false;
		}

		bytes[0] == b'M' && bytes[1] == b'U' && bytes[2] == b'S' && bytes[3] == 0x1A
	}
}
