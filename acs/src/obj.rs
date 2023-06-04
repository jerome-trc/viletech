//! .o file format reader.
//!
//! Assume all code within is derived wholly or in part from GZDoom-original source
//! unless explicitly stated otherwise.

use std::borrow::Cow;

use byteorder::{ByteOrder, LittleEndian};
use util::{io::TCursor, Id8};

use crate::AsciiId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
	Old,
	Enhanced,
	LittleEnhanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LibraryId(u32);

impl LibraryId {
	const SHIFT: u32 = 20;

	#[must_use]
	fn new(inner: u32) -> Self {
		Self(inner << Self::SHIFT)
	}
}

#[derive(Debug)]
pub struct Context {
	pub hexen: bool,
	/// GZDoom derives this from the position of a newly-created behaviour
	/// in a per-level array containing all behaviors.
	pub library_id: u32,
	pub module_name: Id8,
}

#[derive(Debug)]
pub enum Error {
	/// The object was not at least 32 bytes long.
	Undersize(usize),
	/// The first 3 bytes of the object did not match the ASCII characters `A`, `C`, `S`.
	MagicNumber([u8; 4]),
	/// The last byte of the object's 4-byte magic number was not 0, ASCII `E`, or ASCII `e`.
	UnknownFormat(u8),
}

pub fn read(ctx: Context, bytes: Cow<[u8]>) -> Result<(), Error> {
	const PRETAG_ACSE: AsciiId = AsciiId::from_chars(b'A', b'C', b'S', b'E');
	const PRETAG_ACSLE: AsciiId = AsciiId::from_chars(b'A', b'C', b'S', b'e');

	// (GZ)
	// Any behaviors smaller than 32 bytes cannot possibly contain anything useful.
	// (16 bytes for a completely empty behavior + 12 bytes for one script header
	//  + 4 bytes for `PCD_TERMINATE` for an old-style object. A new-style object
	// has 24 bytes if it is completely empty. An empty SPTR chunk adds 8 bytes.)
	if bytes.len() < 32 {
		return Err(Error::Undersize(bytes.len()));
	}

	if bytes[0..3] != [b'A', b'C', b'S'] {
		return Err(Error::MagicNumber([bytes[0], bytes[1], bytes[2], bytes[3]]));
	}

	let mut format = match bytes[3] {
		0 => Format::Old,
		b'E' => Format::Enhanced,
		b'e' => Format::LittleEnhanced,
		other => return Err(Error::UnknownFormat(other)),
	}; // TODO: Assign library ID, module name.

	let lib_id = LibraryId::new(ctx.library_id);

	let mut bytes = bytes.into_owned();
	let dir_offs = LittleEndian::read_u32(&bytes[4..8]) as usize;
	let mut chunks;
	let localize;

	if format == Format::Old {
		let pretag: AsciiId = TCursor::new(&bytes, dir_offs as u64)
			.peek_u32_offs(-1)
			.into();

		chunks = bytes.len();

		// (GZ) Check for redesigned ACSE/ACSe.
		if dir_offs >= 6 * 4 && (pretag == PRETAG_ACSE || pretag == PRETAG_ACSLE) {
			if pretag == PRETAG_ACSE {
				format = Format::Enhanced;
			} else if pretag == PRETAG_ACSLE {
				format = Format::LittleEnhanced;
			}

			chunks = TCursor::new(&bytes, dir_offs as u64).peek_u32_offs(-2) as usize;
			// (GZ) Forget about the compatibility cruft at the end of the lump.
			bytes.truncate(dir_offs - 8);
		}

		localize = false;
	} else {
		chunks = dir_offs;
		localize = true;
	}

	// Make it harder to unintentionally mutate this.
	let bytes = bytes.into_boxed_slice();

	// TODO: Load scripts directory.

	if format == Format::Old {
		// TODO:
		// - Locate and un-escape string table.
		// - Check if localization is necessary.
	} else {
		// TODO: Unencrypt and un-escape string table.
	}

	if format == Format::Old {
		// TODO: Initialize map variables.
	} else {
		// TODO:
		// - Load functions.
		// - Load function local arrays.
		// - Load jump points.
		// - Initialize map variables.
		// - Create and initialize arrays.
		// - Set up array pointers.
		// - Tag library ID to map variables initialized with strings.
		// - Load required libraries.

		#[cfg(feature = "parallel")]
		let _ = load_par(BehaviorReader { bytes: &bytes, chunks });
	}

	let _ = (format, localize, chunks, lib_id);

	unimplemented!()
}

#[cfg(feature = "parallel")]
#[must_use]
fn load_par(reader: BehaviorReader) -> Artifacts {
	let (f, e) = rayon::join(|| {
		read_functions(reader.clone())
	}, || {
		read_jump_points(reader.clone())
	});

	Artifacts { functions: f.unwrap_or_default(), jump_pts: e.unwrap_or_default() }
}

#[must_use]
fn read_functions(reader: BehaviorReader) -> Option<Vec<Function>> {
	let Some(mut c8) = reader.find_chunk(AsciiId::from_bstr(b"FUNC")) else { return None; };
	let c32 = c8.to_tcursor32();
	let len = c32.peek_u32_offs_le(1) / 8;
	let mut ret = vec![];

	c8.advance(8);

	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct RawFunc {
		arg_count: u8,
		local_count: u8,
		has_ret_val: u8,
		import_num: u8,
		address: u32,
	}

	for _ in 0..len {
		let raw = c8.read_from_bytes::<RawFunc>();

		ret.push(Function {
			arg_count: raw.arg_count,
			has_ret_val: raw.has_ret_val,
			import_num: raw.import_num,
			local_count: raw.local_count as u32,
			address: u32::from_le(raw.address),
			local_arrays: vec![],
		});
	}

	Some(ret)
}

#[must_use]
fn read_jump_points(reader: BehaviorReader) -> Option<Vec<JumpPoint>> {
	let Some(c8) = reader.find_chunk(AsciiId::from_bstr(b"JUMP")) else { return None; };
	let c32 = c8.to_tcursor32();
	let len = c32.peek_u32_offs_le(1);
	let mut ret = vec![];

	for i in (0..len).step_by(4) {
		let jmpt = c32.peek_u32_offs_le(2 + (i as isize) / 4);
		ret.push(JumpPoint(jmpt));
	}

	Some(ret)
}

#[derive(Debug, Clone)]
struct BehaviorReader<'b> {
	/// The entire byte content of the ACS object file.
	bytes: &'b [u8],
	/// An index into `bytes`.
	chunks: usize,
}

impl<'b> BehaviorReader<'b> {
	#[must_use]
	fn find_chunk(&self, id: AsciiId) -> Option<TCursor<u8>> {
		let mut ret = TCursor::new(&self.bytes, self.chunks as u64);

		while ret.pos() < (self.bytes.len() as u64) {
			let mut c32 = ret.to_tcursor32();

			if c32.peek_u32() == id.0 {
				return Some(ret);
			}

			ret.advance(c32.advance(1).peek_u32_le() as u64);
			ret.advance(8);
		}

		None
	}

	#[must_use]
	fn next_chunk(&'b self, mut chunk: TCursor<'b, u8>) -> Option<TCursor<u8>> {
		let id = chunk.to_tcursor32().peek_u32();
		let adv = chunk.to_tcursor32().advance(1).peek_u32_le() as u64;
		chunk.advance(adv + 8);

		while chunk.pos() < (self.bytes.len() as u64) {
			let mut c32 = chunk.to_tcursor32();

			if c32.peek_u32() == id {
				return Some(chunk);
			}

			chunk.advance(c32.advance(1).peek_u32_le() as u64);
			chunk.advance(8);
		}

		None
	}
}

#[derive(Debug)]
struct Artifacts {
	functions: Vec<Function>,
	jump_pts: Vec<JumpPoint>,
}

#[derive(Debug)]
struct Function {
	arg_count: u8,
	has_ret_val: u8, // Q: Should this be a boolean?
	import_num: u8,
	local_count: u32,
	address: u32,
	local_arrays: Vec<LocalArrayInfo>,
}

#[derive(Debug)]
struct LocalArrayInfo {
	size: usize,
	offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct JumpPoint(u32);
