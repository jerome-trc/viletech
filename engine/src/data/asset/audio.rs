//! Sound and music assets.

use byteorder::{ByteOrder, LittleEndian};
use kira::sound::static_sound::StaticSoundData;

use crate::audio::MidiData;

use super::AssetHeader;

#[derive(Debug)]
pub struct Audio {
	pub header: AssetHeader,
	pub data: AudioData,
}

#[derive(Debug)]
pub enum AudioData {
	Midi(MidiData),
	Waveform(StaticSoundData),
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
