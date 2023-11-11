//! Implementation details of [`VirtualFs::mount`].

use std::{
	fs::File,
	io::{Cursor, Read, Seek, SeekFrom},
	path::Path,
	sync::Arc,
};

use parking_lot::RwLock;
use util::SmallString;
use zip_structs::{zip_central_directory::ZipCDEntry, zip_eocd::ZipEOCD};

use super::{
	detail, Compression, Error, FolderSlot, MountFormat, MountInfo, Reader, Slot, VFile, VFolder,
	VPath, VPathBuf, VirtualFs,
};

pub(super) fn mount(vfs: &mut VirtualFs, real: &Path, mpoint: &str) -> Result<MountInfo, Error> {
	if real.is_dir() {
		let oslot = mount_dir(vfs, real, vfs.root)?;

		return Ok(MountInfo {
			real_path: real.to_path_buf(),
			mount_point: VPathBuf::new(format!("/{mpoint}")),
			root: Slot::Folder(oslot),
			format: MountFormat::Directory,
		});
	}

	let mut fh = std::fs::File::open(real).map_err(Error::FileOpen)?;

	if let Ok(w_reader) = wadload::DirReader::new(&mut fh) {
		let oslot = mount_wad_file(vfs, mpoint, vfs.root, w_reader)?;

		return Ok(MountInfo {
			real_path: real.to_path_buf(),
			mount_point: VPathBuf::new(format!("/{mpoint}")),
			root: Slot::Folder(oslot),
			format: MountFormat::Wad,
		});
	}

	fh.seek(SeekFrom::Start(0)).map_err(Error::Seek)?;
	let (magic, len) = magic_and_length(&mut fh)?;

	if util::io::is_zip(&magic) {
		let oslot = mount_zip_file(vfs, fh, mpoint, vfs.root)?;

		return Ok(MountInfo {
			real_path: real.to_path_buf(),
			mount_point: VPathBuf::new(format!("/{mpoint}")),
			root: Slot::Folder(oslot),
			format: MountFormat::Zip,
		});
	}

	let islot = vfs.files.insert(VFile {
		name: real.file_name().unwrap().to_string_lossy().into(),
		parent: vfs.root,
		reader: Arc::new(RwLock::new(Reader::File(fh))),
		span: 0..(len as u32),
		compression: Compression::None,
	});

	vfs.folders[vfs.root].files.push(islot);

	Ok(MountInfo {
		real_path: real.to_path_buf(),
		mount_point: VPathBuf::new(format!("/{mpoint}")),
		root: Slot::File(islot),
		format: MountFormat::Uncompressed,
	})
}

fn mount_dir(
	vfs: &mut VirtualFs,
	real: &Path,
	parent_slot: FolderSlot,
) -> Result<FolderSlot, Error> {
	let d_reader = std::fs::read_dir(real).map_err(Error::DirRead)?;

	let oslot = vfs.folders.insert(VFolder {
		name: real.file_name().unwrap().to_string_lossy().into(),
		parent: Some(parent_slot),
		files: vec![],
		subfolders: vec![],
	});

	for result in d_reader {
		let d_ent = result.map_err(Error::DirRead)?;
		let path = d_ent.path();

		if path.is_dir() {
			let _ = mount_dir(vfs, &path, oslot)?;
			continue;
		}

		let mut fh = std::fs::File::open(&path).map_err(Error::FileOpen)?;
		let (magic, len) = magic_and_length(&mut fh)?;

		let mut name = SmallString::from(path.file_name().unwrap().to_string_lossy());
		name.make_ascii_lowercase();

		if wad_extension(name.as_str()) && wad_magic(&magic) {
			let mut bytes = vec![];
			fh.read_to_end(&mut bytes).map_err(Error::FileRead)?;
			let _ = mount_wad_blob(vfs, name.as_str(), oslot, bytes)?;
			continue;
		}

		let islot = vfs.files.insert(VFile {
			name,
			parent: oslot,
			reader: Arc::new(RwLock::new(Reader::File(fh))),
			span: 0..(len as u32),
			compression: Compression::None,
		});

		vfs.folders[oslot].files.push(islot);
	}

	vfs.folders[parent_slot].subfolders.push(oslot);

	Ok(oslot)
}

fn mount_wad_file(
	vfs: &mut VirtualFs,
	mpoint: &str,
	parent_slot: FolderSlot,
	w_reader: wadload::DirReader<&mut File>,
) -> Result<FolderSlot, Error> {
	let rfh = w_reader
		.get_ref()
		.try_clone()
		.map_err(Error::FileHandleClone)?;

	let arc = Arc::new(RwLock::new(Reader::File(rfh)));

	let oslot = vfs.folders.insert(VFolder {
		name: SmallString::from(mpoint),
		parent: Some(parent_slot),
		files: vec![],
		subfolders: vec![],
	});

	let folder = &mut vfs.folders[oslot];

	for result in w_reader {
		let w_ent = result.map_err(Error::Wad)?;

		let mut name = SmallString::from(w_ent.name.as_str());
		name.make_ascii_lowercase();

		let islot = vfs.files.insert(VFile {
			name,
			parent: oslot,
			reader: arc.clone(),
			span: (w_ent.span.start as u32)..(w_ent.span.end as u32),
			compression: Compression::None,
		});

		folder.files.push(islot);
	}

	vfs.folders[parent_slot].subfolders.push(oslot);

	Ok(oslot)
}

fn mount_wad_blob(
	vfs: &mut VirtualFs,
	mpoint: &str,
	parent_slot: FolderSlot,
	bytes: Vec<u8>,
) -> Result<FolderSlot, Error> {
	let arc = Arc::new(RwLock::new(Reader::Memory(bytes)));
	let guard = arc.read();

	let Reader::Memory(blob) = std::ops::Deref::deref(&guard) else {
		unreachable!()
	};

	let cursor = Cursor::new(blob.as_slice());
	let w_reader = wadload::DirReader::new(cursor).map_err(Error::Wad)?;

	let oslot = vfs.folders.insert(VFolder {
		name: SmallString::from(mpoint),
		parent: Some(parent_slot),
		files: vec![],
		subfolders: vec![],
	});

	let folder = &mut vfs.folders[oslot];

	for result in w_reader {
		let w_ent = result.map_err(Error::Wad)?;

		let mut name = SmallString::from(w_ent.name.as_str());
		name.make_ascii_lowercase();

		let islot = vfs.files.insert(VFile {
			name,
			parent: oslot,
			reader: arc.clone(),
			span: (w_ent.span.start as u32)..(w_ent.span.end as u32),
			compression: Compression::None,
		});

		folder.files.push(islot);
	}

	vfs.folders[parent_slot].subfolders.push(oslot);

	Ok(oslot)
}

fn mount_zip_file(
	vfs: &mut VirtualFs,
	mut fh: File,
	mpoint: &str,
	parent_slot: FolderSlot,
) -> Result<FolderSlot, Error> {
	let rfh = fh.try_clone().map_err(Error::FileHandleClone)?;

	let arc = Arc::new(RwLock::new(Reader::File(rfh)));

	let eocd = ZipEOCD::from_reader(&mut fh).map_err(Error::Zip)?;
	let entries = ZipCDEntry::all_from_eocd(&mut fh, &eocd).map_err(Error::Zip)?;

	let oslot = vfs.folders.insert(VFolder {
		name: SmallString::from(mpoint),
		parent: Some(vfs.root),
		files: vec![],
		subfolders: vec![],
	});

	for entry in entries {
		let compression = match entry.compression_method {
			0 => Compression::None,
			8 => Compression::Deflate,
			12 => Compression::Bzip2,
			14 => Compression::Lzma,
			93 => Compression::Zstd,
			95 => Compression::Xz,
			_ => unimplemented!(),
		};

		let epath = String::from_utf8_lossy(&entry.file_name_raw);

		let components = epath
			.split('/')
			.filter(|pcomp| pcomp.contains(|c| char::is_ascii_alphanumeric(&c)));

		let Some(name) = components.clone().last() else {
			continue;
		};

		let eparent = build_zip_dir_structure(vfs, oslot, components, name);

		let start = entry.local_header_position
			+ 4 + 22 + 2 + 2
			+ (entry.file_name_length as u32)
			+ (entry.extra_field_length as u32);

		let span = start..(start + entry.compressed_size);

		if wad_extension(name) {
			let mut compressed = vec![0; span.len()];
			fh.seek(SeekFrom::Start(start as u64))
				.map_err(Error::Seek)?;
			fh.read_exact(&mut compressed).map_err(Error::FileRead)?;
			let bytes = detail::decompress(compressed, compression)?;

			if wad_magic(&bytes[0..8]) {
				let s = mount_wad_blob(vfs, name, eparent, bytes)?;
				vfs.folders[eparent].subfolders.push(s);
				continue;
			}
		}

		let islot = vfs.files.insert(VFile {
			name: SmallString::from(name),
			parent: eparent,
			reader: arc.clone(),
			span,
			compression,
		});

		vfs.folders[eparent].files.push(islot);
	}

	vfs.folders[parent_slot].subfolders.push(oslot);

	Ok(oslot)
}

#[must_use]
fn build_zip_dir_structure<'s>(
	vfs: &mut VirtualFs,
	zip_slot: FolderSlot,
	components: impl Iterator<Item = &'s str>,
	entry_name: &str,
) -> FolderSlot {
	let mut eparent = zip_slot;

	for comp in components {
		if std::ptr::eq(comp, entry_name) {
			return eparent;
		}

		let opt = vfs.folders[eparent]
			.subfolders
			.iter()
			.copied()
			.find(|sfslot| vfs.folders[*sfslot].name == comp);

		eparent = opt.unwrap_or_else(|| {
			let s = vfs.folders.insert(VFolder {
				name: SmallString::from(comp),
				parent: Some(eparent),
				files: vec![],
				subfolders: vec![],
			});

			vfs.folders[eparent].subfolders.push(s);

			s
		});
	}

	zip_slot
}

#[must_use]
fn wad_extension(file_name: &str) -> bool {
	VPath::new(file_name)
		.extension()
		.is_some_and(|ext| ext.eq_ignore_ascii_case("wad"))
}

#[must_use]
fn wad_magic(magic: &[u8]) -> bool {
	magic.len() >= 4
		&& matches!(
			magic[0..4],
			[b'P', b'W', b'A', b'D'] | [b'I', b'W', b'A', b'D']
		)
}

fn magic_and_length(fh: &mut File) -> Result<([u8; 8], u64), Error> {
	let mut buf = [0; 8];
	fh.read_exact(&mut buf).map_err(Error::FileRead)?;
	let r = fh.seek(SeekFrom::End(0)).map_err(Error::Seek)?;
	Ok((buf, r))
}
