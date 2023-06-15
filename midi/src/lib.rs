//! # Sunrider
//!
//! Sunrider is the crate representing VileTech's code for reading, writing, and
//! playing the [MIDI](https://doomwiki.org/wiki/MIDI) and
//! [DMXMUS](https://doomwiki.org/wiki/MUS) file formats.

pub mod mus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
	Midi,
	Hmi,
	Xmi,
	DmxMus,
	Mids,
}

impl Format {
	/// From ZMusic.
	///
	/// For licensing information, see the repository's ATTRIB.md file.
	#[must_use]
	pub fn deduce(bytes: &[u8]) -> Option<Format> {
		use util::Ascii4;

		if bytes.len() < 12 {
			return None;
		}

		if bytes[0] == b'M' && bytes[1] == b'U' && bytes[2] == b'S' && bytes[3] == 0x1A {
			return Some(Self::DmxMus);
		}

		let m0 = Ascii4::from_bytes(bytes[0], bytes[1], bytes[2], bytes[3]);
		let m1 = Ascii4::from_bytes(bytes[4], bytes[5], bytes[6], bytes[7]);
		let m2 = Ascii4::from_bytes(bytes[8], bytes[9], bytes[10], bytes[11]);

		if m0 == Ascii4::from_bstr(b"HMI-")
			&& m1 == Ascii4::from_bstr(b"MIDI")
			&& m2 == Ascii4::from_bstr(b"SONG")
		{
			return Some(Self::Hmi);
		}

		if m0 == Ascii4::from_bstr(b"HMIM") && m1 == Ascii4::from_bstr(b"IDIP") {
			return Some(Self::Hmi);
		}

		if m0 == Ascii4::from_bstr(b"FORM") && m2 == Ascii4::from_bstr(b"XDIR") {
			return Some(Self::Xmi);
		}

		if (m0 == Ascii4::from_bstr(b"CAT ") || m0 == Ascii4::from_bstr(b"FORM"))
			&& m2 == Ascii4::from_bstr(b"XMID")
		{
			return Some(Self::Xmi);
		}

		if m0 == Ascii4::from_bstr(b"RIFF") && m2 == Ascii4::from_bstr(b"MIDS") {
			return Some(Self::Mids);
		}

		if m0 == Ascii4::from_bstr(b"MThd") {
			return Some(Self::Midi);
		}

		None
	}
}
