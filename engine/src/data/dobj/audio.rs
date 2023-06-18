//! Sound and music assets.

use byteorder::{ByteOrder, LittleEndian};
use kira::sound::static_sound::StaticSoundData;

use crate::audio::MidiData;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // TODO: `MidiData` will eventually reach size parity.
pub enum Audio {
	Midi(MidiData),
	Waveform(StaticSoundData),
}

impl Audio {
	/// See <https://doomwiki.org/wiki/MUS>.
	#[must_use]
	pub fn is_dmxmus(bytes: &[u8]) -> bool {
		if bytes.len() < 4 {
			return false;
		}

		bytes[0] == b'M' && bytes[1] == b'U' && bytes[2] == b'S' && bytes[3] == 0x1A
	}

	/// Source: <https://docs.rs/infer/0.13.0/src/infer/matchers/audio.rs.html#50-52>
	#[must_use]
	pub fn is_flac(bytes: &[u8]) -> bool {
		bytes.len() > 3 && matches!(bytes, &[0x66, 0x4c, 0x61, 0x43])
	}

	/// Source: <https://docs.rs/infer/0.13.0/src/infer/matchers/audio.rs.html#7-12>
	#[must_use]
	pub fn is_mp3(bytes: &[u8]) -> bool {
		bytes.len() > 2
			&& matches!(bytes, &[])
			&& ((bytes[0] == 0x49 && bytes[1] == 0x44 && bytes[2] == 0x33) // ID3v2
		// (INFER) Final bit (has crc32) may be or may not be set.
		|| (bytes[0] == 0xFF && bytes[1] == 0xFB))
	}

	/// Source: <https://docs.rs/infer/0.13.0/src/infer/matchers/audio.rs.html#28-30>
	#[must_use]
	pub fn is_ogg(bytes: &[u8]) -> bool {
		bytes.len() > 3 && matches!(bytes, &[0x4f, 0x67, 0x67, 0x53])
	}

	/// Adapted from SLADE's `DoomPCSpeakerDataFormat::isThisFormat`.
	#[must_use]
	pub fn is_pc_speaker_sound(bytes: &[u8]) -> bool {
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

	/// Source: <https://docs.rs/infer/0.13.0/src/infer/matchers/audio.rs.html#55-65>
	#[must_use]
	pub fn is_wav(bytes: &[u8]) -> bool {
		bytes.len() > 11
			&& bytes[0] == 0x52
			&& bytes[1] == 0x49
			&& bytes[2] == 0x46
			&& bytes[3] == 0x46
			&& bytes[8] == 0x57
			&& bytes[9] == 0x41
			&& bytes[10] == 0x56
			&& bytes[11] == 0x45
	}
}
