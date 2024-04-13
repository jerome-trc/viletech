//! Internal implementation details not related to mounting.

use std::{
	borrow::Cow,
	fs::File,
	io::{Read, Seek, SeekFrom},
	ops::Range,
	sync::Arc,
};

use flate2::read::DeflateDecoder;
use parking_lot::RwLock;

use super::{Error, FolderSlot, VirtualFs};

pub(super) fn path_append(vfs: &VirtualFs, buf: &mut String, slot: FolderSlot) {
	let folder = &vfs.folders[slot];

	if slot != vfs.root {
		buf.insert(0, '/');
	}

	buf.insert_str(0, &folder.name);

	if let Some(p) = folder.parent {
		path_append(vfs, buf, p);
	}
}

pub(super) fn decompress(bytes: Cow<[u8]>, compression: Compression) -> Result<Cow<[u8]>, Error> {
	match compression {
		Compression::None => Ok(bytes),
		Compression::Deflate => {
			let mut deflater = DeflateDecoder::new(&bytes[..]);
			let mut decompressed = vec![];

			deflater
				.read_to_end(&mut decompressed)
				.map_err(Error::Decompress)?;

			Ok(Cow::Owned(decompressed))
		}
		Compression::Bzip2 | Compression::Lzma | Compression::Xz | Compression::Zstd => {
			unimplemented!()
		}
	}
}

#[derive(Debug)]
pub(crate) enum Reader {
	/// e.g. lump in a WAD, or entry in a zip archive.
	File(File),
	Memory(Vec<u8>),
	/// e.g. entry in a zip archive nested within another zip archive.
	_Super(ReaderLayer),
}

impl Reader {
	pub(super) fn read(
		&mut self,
		span: Range<usize>,
		compression: Compression,
	) -> Result<Cow<[u8]>, Error> {
		let bytes = match self {
			Self::File(ref mut fh) => Cow::Owned(Self::read_from_file(fh, span)?),
			Self::Memory(bytes) => Cow::Borrowed(bytes.as_slice()),
			Self::_Super(layer) => {
				let mut guard = layer.parent.write();
				let cow = guard.read(layer.span.clone(), layer.compression)?;
				Cow::Owned(cow.into_owned())
			}
		};

		decompress(bytes, compression)
	}

	pub(super) fn read_from_file(fh: &mut File, span: Range<usize>) -> Result<Vec<u8>, Error> {
		fh.seek(SeekFrom::Start(span.start as u64))
			.map_err(Error::Seek)?;
		let mut bytes = vec![0; span.len()];
		fh.read_exact(&mut bytes).map_err(Error::FileRead)?;
		Ok(bytes)
	}
}

#[derive(Debug)]
pub(crate) struct ReaderLayer {
	pub(crate) parent: Arc<RwLock<Reader>>,
	pub(crate) span: Range<usize>,
	pub(crate) compression: Compression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Compression {
	/// Always the case for WAD lumps.
	None,
	Bzip2,
	Deflate,
	Lzma,
	Xz,
	Zstd,
}
