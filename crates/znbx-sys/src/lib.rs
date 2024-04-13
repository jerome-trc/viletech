//! # znbx-sys
//!
//! Automatically-generated FFI bindings for ZNBX, a fork of (G)ZDoom's internal
//! binary space partition node tree builder, [ZDBSP].
//!
//! [ZDBSP]: https://zdoom.org/wiki/ZDBSP

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Tests for verifying the correctness of changes made to upstream ZDBSP,
/// as well the correctness of the wrapper defined by znbx.cpp.
#[cfg(test)]
mod test {
	use std::{
		ffi::c_void,
		io::Cursor,
		path::{Path, PathBuf},
	};

	use super::*;

	#[test]
	fn vanilla_smoke() {
		let level = load_level(None);
		let mut hash_in = HashInput::default();

		unsafe {
			let p = znbx_processor_new_vanilla(level);
			znbx_processor_run(p, std::ptr::null());

			let magic_ptr = znbx_processor_magicnumber(p, true as u8);
			let magic = std::ptr::read::<[i8; 4]>(magic_ptr.cast());
			assert_eq!(magic, [b'Z' as i8, b'N' as i8, b'O' as i8, b'D' as i8]);

			znbx_processor_nodes_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			znbx_processor_segs_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seg_callback),
			);

			znbx_processor_ssectors_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssector_callback),
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.nodes.as_slice())),
				"375e670aef63eddb364b41b40f19ee02"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.segs.as_slice())),
				"9bc66ebed4271c73bb938b76b20f204c"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"41496992928328ea481f60f1cbb13dc5"
			);

			let blockmap = znbx_processor_blockmap(p);
			let blockmap = std::slice::from_raw_parts(blockmap.ptr, blockmap.len);
			let blockmap_bytes = blockmap.align_to();

			assert_eq!(
				format!("{:#?}", md5::compute(blockmap_bytes.1)),
				"ca8320b3126bf740d558220f802a3f71"
			);

			znbx_processor_destroy(p);
		}
	}

	#[test]
	fn extended_smoke() {
		let level = load_level(None);
		let mut hash_in = HashInput::default();

		unsafe {
			let p = znbx_processor_new_vanilla(level);
			znbx_processor_run(p, std::ptr::null());

			for b in (znbx_processor_vertsorig_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			for b in (znbx_processor_vertsnewx_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			znbx_processor_vertsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(vertx_callback),
			);

			for b in (znbx_processor_ssectors_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_ssectorsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_callback),
			);

			for b in (znbx_processor_segs_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_segsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segx_callback),
			);

			for b in (znbx_processor_nodes_count(p) as u32).to_le_bytes() {
				hash_in.nodes.push(b);
			}

			znbx_processor_nodesx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodex_callback),
			);

			let mut all_bytes = vec![b'X', b'N', b'O', b'D'];

			all_bytes.append(&mut hash_in.verts);
			all_bytes.append(&mut hash_in.subsectors);
			all_bytes.append(&mut hash_in.segs);
			all_bytes.append(&mut hash_in.nodes);

			let checksum = format!("{:#?}", md5::compute(all_bytes.as_slice()));
			assert_eq!(checksum, "30025de1f1cf2a091cd7e2c92ea0af88");

			znbx_processor_destroy(p);
		}
	}

	#[test]
	fn glnodes_smoke() {
		let level = load_level(None);
		let mut hash_in = HashInput::default();

		unsafe {
			let pcfg = znbx_ProcessConfig {
				flags: znbx_processflags_default() | znbx_ProcessFlags_ZNBX_PROCF_BUILDGLNODES,
				reject_mode: znbx_rejectmode_default(),
				blockmap_mode: znbx_blockmapmode_default(),
			};

			let p = znbx_processor_new_vanilla(level);
			znbx_processor_configure(p, std::ptr::addr_of!(pcfg));
			znbx_processor_run(p, std::ptr::null());

			znbx_processor_nodesgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			znbx_processor_segsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seggl_callback),
			);

			znbx_processor_ssectorsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssector_callback),
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.nodes.as_slice())),
				"f1d971b1b0188c4cdbd32b7b4d1123f1"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.segs.as_slice())),
				"dfed7b623c2136bc727562d958a4c9b3"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"8aa841c49b27f02232bede64205c8790"
			);

			znbx_processor_destroy(p);
		}
	}

	#[test]
	fn glnodes_conform() {
		let level = load_level(None);
		let mut hash_in = HashInput::default();

		unsafe {
			let pcfg = znbx_ProcessConfig {
				flags: znbx_processflags_default()
					| znbx_ProcessFlags_ZNBX_PROCF_BUILDGLNODES
					| znbx_ProcessFlags_ZNBX_PROCF_CONFORMNODES,
				reject_mode: znbx_rejectmode_default(),
				blockmap_mode: znbx_blockmapmode_default(),
			};

			let p = znbx_processor_new_vanilla(level);
			znbx_processor_configure(p, std::ptr::addr_of!(pcfg));
			znbx_processor_run(p, std::ptr::null());

			znbx_processor_nodes_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			znbx_processor_segs_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seg_callback),
			);

			znbx_processor_ssectors_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssector_callback),
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.nodes.as_slice())),
				"885acd04ba60856b66b7446099e1930b"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.segs.as_slice())),
				"6c0c2c5b9620731ee41f62ba02950fb7"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"1a4e5ddf72e0a54899a0d6019a07aa5c"
			);

			hash_in.nodes.clear();
			hash_in.segs.clear();
			hash_in.subsectors.clear();

			znbx_processor_nodesgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			znbx_processor_segsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seggl_callback),
			);

			znbx_processor_ssectorsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssector_callback),
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.nodes.as_slice())),
				"f1d971b1b0188c4cdbd32b7b4d1123f1"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.segs.as_slice())),
				"dfed7b623c2136bc727562d958a4c9b3"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"8aa841c49b27f02232bede64205c8790"
			);

			znbx_processor_destroy(p);
		}
	}

	#[test]
	fn glv5_smoke() {
		let level = load_level(None);
		let mut hash_in = HashInput::default();

		unsafe {
			let pcfg = znbx_ProcessConfig {
				flags: znbx_processflags_default()
					| znbx_ProcessFlags_ZNBX_PROCF_BUILDGLNODES
					| znbx_ProcessFlags_ZNBX_PROCF_V5GL,
				reject_mode: znbx_rejectmode_default(),
				blockmap_mode: znbx_blockmapmode_default(),
			};

			let p = znbx_processor_new_vanilla(level);
			znbx_processor_configure(p, std::ptr::addr_of!(pcfg));
			znbx_processor_run(p, std::ptr::null());

			znbx_processor_nodesx_v5_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodexo_callback),
			);

			znbx_processor_segsglx_v5_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segglx_v5_callback),
			);

			znbx_processor_ssectorsx_v5_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_v5_callback),
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.nodes.as_slice())),
				"b3696762fda6b6d433d69e9a63c27823"
			);

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.segs.as_slice())),
				"b2ed4dcdb90afe7da4315e4a8d05a01f"
			);

			// Note that this checksum is derived from a `GL_SSECT` lump emitted
			// by a modified ZNBX CLI that fills in padding bytes of subsector
			// records with zeroes.

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"3bc19dc80a5cbe4f704cb696853bc831"
			);

			znbx_processor_destroy(p);
		}
	}

	#[test]
	fn udmf_smoke() {
		let level = load_level_udmf();
		let mut hash_in = HashInput::default();

		unsafe {
			let p = znbx_processor_new_udmf(level);
			znbx_processor_run(p, std::ptr::null());

			for b in (znbx_processor_vertsorig_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			for b in (znbx_processor_vertsnewgl_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			znbx_processor_vertsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(vertx_callback),
			);

			for b in (znbx_processor_ssectorsgl_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_ssectorsglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_callback),
			);

			for b in (znbx_processor_segsglx_count(p) as u32).to_le_bytes() {
				hash_in.segs.push(b);
			}

			znbx_processor_segsglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segglx_callback),
			);

			for b in (znbx_processor_nodesgl_count(p) as u32).to_le_bytes() {
				hash_in.nodes.push(b);
			}

			znbx_processor_nodesglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodex_callback),
			);

			let magic_ptr = znbx_processor_magicnumber(p, false as u8);
			let magic = std::ptr::read::<[i8; 4]>(magic_ptr.cast());
			assert_eq!(magic, [b'X' as i8, b'G' as i8, b'L' as i8, b'N' as i8]);
			let mut all_bytes = vec![b'X', b'G', b'L', b'N'];

			all_bytes.append(&mut hash_in.verts);
			all_bytes.append(&mut hash_in.subsectors);
			all_bytes.append(&mut hash_in.segs);
			all_bytes.append(&mut hash_in.nodes);

			let checksum = format!("{:#?}", md5::compute(all_bytes.as_slice()));
			assert_eq!(checksum, "39ed77ca24155506b2455a887243c3ef");

			znbx_processor_destroy(p);
		}
	}

	/// This always assumes extended format, since it is for testing correctness
	/// on levels which push the upper boundaries of non-UDMF node-building.
	#[test]
	#[ignore]
	fn user_sample() {
		let Ok(wad_path) = std::env::var("ZNBX_SAMPLE_WAD") else {
			eprintln!("Env. var. `ZNBX_SAMPLE_WAD` not set; skipping user sample test.");
			return;
		};

		let level = load_level(Some(PathBuf::from(wad_path)));

		unsafe {
			let p = znbx_processor_new_vanilla(level);

			let pcfg = znbx_ProcessConfig {
				flags: znbx_processflags_default() | znbx_ProcessFlags_ZNBX_PROCF_BUILDGLNODES,
				reject_mode: znbx_rejectmode_default(),
				blockmap_mode: znbx_blockmapmode_default(),
			};

			znbx_processor_configure(p, std::ptr::addr_of!(pcfg));
			znbx_processor_run(p, std::ptr::null());

			let mut hash_in = HashInput::default();

			for b in (znbx_processor_vertsorig_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			for b in (znbx_processor_vertsnewx_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			znbx_processor_vertsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(vertx_callback),
			);

			for b in (znbx_processor_ssectors_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_ssectorsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_callback),
			);

			for b in (znbx_processor_segs_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_segsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segx_callback),
			);

			for b in (znbx_processor_nodes_count(p) as u32).to_le_bytes() {
				hash_in.nodes.push(b);
			}

			znbx_processor_nodesx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodex_callback),
			);

			let mut all_bytes = vec![b'X', b'N', b'O', b'D'];

			all_bytes.append(&mut hash_in.verts);
			all_bytes.append(&mut hash_in.subsectors);
			all_bytes.append(&mut hash_in.segs);
			all_bytes.append(&mut hash_in.nodes);

			if let Ok(cksum) = std::env::var("ZNBX_SAMPLE_CHECKSUM_NODES") {
				assert_eq!(format!("{:#?}", md5::compute(all_bytes.as_slice())), cksum,);
			}

			let mut hash_in = HashInput::default();

			for b in (znbx_processor_vertsorig_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			for b in (znbx_processor_vertsnewgl_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			znbx_processor_vertsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(vertx_callback),
			);

			for b in (znbx_processor_ssectorsgl_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			znbx_processor_ssectorsglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_callback),
			);

			for b in (znbx_processor_segsglx_count(p) as u32).to_le_bytes() {
				hash_in.segs.push(b);
			}

			znbx_processor_segsglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segglx_callback),
			);

			for b in (znbx_processor_nodesgl_count(p) as u32).to_le_bytes() {
				hash_in.nodes.push(b);
			}

			znbx_processor_nodesglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodex_callback),
			);

			let mut all_bytes = vec![b'X', b'G', b'L', b'N'];

			all_bytes.append(&mut hash_in.verts);
			all_bytes.append(&mut hash_in.subsectors);
			all_bytes.append(&mut hash_in.segs);
			all_bytes.append(&mut hash_in.nodes);

			if let Ok(cksum) = std::env::var("ZNBX_SAMPLE_CHECKSUM_NODESGL") {
				assert_eq!(format!("{:#?}", md5::compute(all_bytes.as_slice())), cksum,);
			}

			if let Ok(cksum) = std::env::var("ZNBX_SAMPLE_CHECKSUM_BLOCKMAP") {
				let blockmap = znbx_processor_blockmap(p);
				let blockmap = std::slice::from_raw_parts(blockmap.ptr, blockmap.len);
				let blockmap_bytes = blockmap.align_to();
				assert_eq!(format!("{:#?}", md5::compute(blockmap_bytes.1)), cksum,);
			}

			znbx_processor_destroy(p);
		}
	}

	// Details and helpers /////////////////////////////////////////////////////

	#[must_use]
	fn load_level(path: Option<PathBuf>) -> znbx_Level {
		let path = path.unwrap_or_else(|| {
			Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad")
		});

		let wad_bytes = std::fs::read(&path).unwrap();
		let mut reader = wadload::Reader::new(Cursor::new(wad_bytes)).unwrap();

		while reader
			.next()
			.is_some_and(|result| result.is_ok_and(|(d, _)| !d.name.eq_ignore_ascii_case("MAP01")))
		{}

		let b_things = reader.next().unwrap().unwrap();
		let b_linedefs = reader.next().unwrap().unwrap();
		let b_sidedefs = reader.next().unwrap().unwrap();
		let b_vertexes = reader.next().unwrap().unwrap();
		let _b_segs = reader.next().unwrap().unwrap();
		let _b_ssectors = reader.next().unwrap().unwrap();
		let _b_nodes = reader.next().unwrap().unwrap();
		let b_sectors = reader.next().unwrap().unwrap();

		let s_things = znbx_SliceU8 {
			ptr: b_things.1.as_ptr(),
			len: b_things.1.len(),
		};

		let s_linedefs = znbx_SliceU8 {
			ptr: b_linedefs.1.as_ptr(),
			len: b_linedefs.1.len(),
		};

		let s_sidedefs = znbx_SliceU8 {
			ptr: b_sidedefs.1.as_ptr(),
			len: b_sidedefs.1.len(),
		};

		let s_vertices = znbx_SliceU8 {
			ptr: b_vertexes.1.as_ptr(),
			len: b_vertexes.1.len(),
		};

		let s_sectors = znbx_SliceU8 {
			ptr: b_sectors.1.as_ptr(),
			len: b_sectors.1.len(),
		};

		std::mem::forget(b_things.1);
		std::mem::forget(b_linedefs.1);
		std::mem::forget(b_sidedefs.1);
		std::mem::forget(b_vertexes.1);
		std::mem::forget(b_sectors.1);

		znbx_Level {
			name: [
				b'M' as i8, b'A' as i8, b'P' as i8, b'0' as i8, b'1' as i8, 0, 0, 0, 0,
			],
			things: s_things,
			vertices: s_vertices,
			linedefs: s_linedefs,
			sidedefs: s_sidedefs,
			sectors: s_sectors,
		}
	}

	#[must_use]
	fn load_level_udmf() -> znbx_LevelUdmf {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/udmf.wad");
		let wad_bytes = std::fs::read(&path).unwrap();
		let mut reader = wadload::Reader::new(Cursor::new(wad_bytes)).unwrap();

		while reader
			.next()
			.is_some_and(|result| result.is_ok_and(|(d, _)| !d.name.eq_ignore_ascii_case("MAP01")))
		{}

		let b_textmap = reader.next().unwrap().unwrap();

		let s_textmap = znbx_SliceU8 {
			ptr: b_textmap.1.as_ptr(),
			len: b_textmap.1.len(),
		};

		std::mem::forget(b_textmap.1);

		znbx_LevelUdmf {
			name: [
				b'M' as i8, b'A' as i8, b'P' as i8, b'0' as i8, b'1' as i8, 0, 0, 0, 0,
			],
			textmap: s_textmap,
		}
	}

	#[derive(Default)]
	struct HashInput {
		segs: Vec<u8>,
		subsectors: Vec<u8>,
		nodes: Vec<u8>,
		verts: Vec<u8>,
	}

	unsafe extern "C" fn node_callback(ctx: *mut c_void, ptr: *const znbx_NodeRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_NodeRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn nodex_callback(ctx: *mut c_void, ptr: *const znbx_NodeEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (((*ptr).x >> 16) as i16).to_le_bytes() {
			(*hash_in).nodes.push(b);
		}

		for b in (((*ptr).y >> 16) as i16).to_le_bytes() {
			(*hash_in).nodes.push(b);
		}

		for b in (((*ptr).dx >> 16) as i16).to_le_bytes() {
			(*hash_in).nodes.push(b);
		}

		for b in (((*ptr).dy >> 16) as i16).to_le_bytes() {
			(*hash_in).nodes.push(b);
		}

		for i in 0..2 {
			for ii in 0..4 {
				for b in ((*ptr).bbox[i][ii] as i16).to_le_bytes() {
					(*hash_in).nodes.push(b);
				}
			}
		}

		for b in (*ptr).children[0].to_le_bytes() {
			(*hash_in).nodes.push(b);
		}

		for b in (*ptr).children[1].to_le_bytes() {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn nodexo_callback(ctx: *mut c_void, ptr: *const znbx_NodeExO) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_NodeExO>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn seg_callback(ctx: *mut c_void, ptr: *const znbx_SegRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_SegRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn ssector_callback(ctx: *mut c_void, ptr: *const znbx_SubsectorRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_SubsectorRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn ssectorx_callback(ctx: *mut c_void, ptr: *const znbx_SubsectorEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (*ptr).num_lines.to_le_bytes() {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn ssectorx_v5_callback(ctx: *mut c_void, ptr: *const znbx_SubsectorEx) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_SubsectorEx>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn segx_callback(ctx: *mut c_void, ptr: *const znbx_SegEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (*ptr).v1.to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in (*ptr).v2.to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in ((*ptr).linedef as u16).to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in ((*ptr).side as u8).to_le_bytes() {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn seggl_callback(ctx: *mut c_void, ptr: *const znbx_SegGl) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_SegGl>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn segglx_callback(ctx: *mut c_void, ptr: *const znbx_SegGlEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (*ptr).v1.to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in (*ptr).partner.to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in ((*ptr).linedef as u16).to_le_bytes() {
			(*hash_in).segs.push(b);
		}

		for b in ((*ptr).side as u8).to_le_bytes() {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn segglx_v5_callback(ctx: *mut c_void, ptr: *const znbx_SegGlEx) {
		const RECORD_SIZE: usize = std::mem::size_of::<znbx_SegGlEx>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn vertx_callback(ctx: *mut c_void, ptr: *const znbx_VertexEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (*ptr).x.to_le_bytes() {
			(*hash_in).verts.push(b);
		}

		for b in (*ptr).y.to_le_bytes() {
			(*hash_in).verts.push(b);
		}
	}
}
