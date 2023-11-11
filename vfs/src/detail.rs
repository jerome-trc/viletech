//! Internal implementation details not related to mounting.

use std::io::Read;

use flate2::read::DeflateDecoder;

use super::{Compression, Error, FolderSlot, VirtualFs};

pub(super) fn path_append(vfs: &VirtualFs, buf: &mut String, slot: FolderSlot) {
	let folder = &vfs.folders[slot];

	buf.insert_str(0, &folder.name);

	if let Some(p) = folder.parent {
		path_append(vfs, buf, p);
	}
}

pub(super) fn decompress(bytes: Vec<u8>, compression: Compression) -> Result<Vec<u8>, Error> {
	match compression {
		Compression::None => Ok(bytes),
		Compression::Deflate => {
			let mut deflater = DeflateDecoder::new(&bytes[..]);
			let mut decompressed = vec![];
			deflater
				.read_to_end(&mut decompressed)
				.map_err(Error::Decompress)?;
			Ok(decompressed)
		}
		Compression::Bzip2 | Compression::Lzma | Compression::Xz | Compression::Zstd => {
			unimplemented!()
		}
	}
}
