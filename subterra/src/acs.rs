//! A reader for the compiled bytecode object files of Raven Software's
//! [Action Code Script](https://doomwiki.org/wiki/ACS).

pub mod func;
pub mod pcode;

#[must_use]
pub fn is_object(bytes: &[u8]) -> bool {
	// (GZ)
	// Any behaviors smaller than 32 bytes cannot possibly contain anything useful.
	// (16 bytes for a completely empty behavior + 12 bytes for one script header
	//  + 4 bytes for `PCD_TERMINATE` for an old-style object. A new-style object
	// has 24 bytes if it is completely empty. An empty SPTR chunk adds 8 bytes.)
	if bytes.len() < 32 {
		return false;
	}

	if bytes[0..3] != [b'A', b'C', b'S'] {
		return false;
	}

	matches!(bytes[3], 0 | b'E' | b'e')
}
