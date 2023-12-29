//! Automatically-generated FFI bindings to zdbsp-rs' C header.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Tests for verifying the correctness of changes made to upstream ZDBSP.
#[cfg(test)]
mod test {
	use std::{ffi::c_void, path::Path};

	use super::*;

	#[test]
	fn vanilla_smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
		let bytes = std::fs::read(&path).unwrap();
		let mut hash_in = HashInput::default();

		unsafe {
			let reader = zdbsp_wadreader_new(bytes.as_ptr());
			let p = zdbsp_processor_new(reader, std::ptr::null());
			zdbsp_processor_run(p, std::ptr::null());

			zdbsp_processor_nodes_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			zdbsp_processor_segs_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seg_callback),
			);

			zdbsp_processor_ssectors_foreach(
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

			zdbsp_processor_destroy(p);
			zdbsp_wadreader_destroy(reader);
		}
	}

	#[test]
	fn glnodes_smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
		let bytes = std::fs::read(&path).unwrap();
		let mut hash_in = HashInput::default();

		unsafe {
			let pcfg = zdbsp_ProcessConfig {
				flags: zdbsp_processflags_default() | zdbsp_ProcessFlags_ZDBSP_PROCF_BUILDGLNODES,
				reject_mode: zdbsp_rejectmode_default(),
				blockmap_mode: zdbsp_blockmapmode_default(),
			};

			let reader = zdbsp_wadreader_new(bytes.as_ptr());
			let p = zdbsp_processor_new(reader, std::ptr::addr_of!(pcfg));
			zdbsp_processor_run(p, std::ptr::null());

			zdbsp_processor_nodesgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			zdbsp_processor_segsgl_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seggl_callback),
			);

			zdbsp_processor_ssectorsgl_foreach(
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

			zdbsp_processor_destroy(p);
			zdbsp_wadreader_destroy(reader);
		}
	}

	#[test]
	fn glv5_smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
		let bytes = std::fs::read(&path).unwrap();
		let mut hash_in = HashInput::default();

		unsafe {
			let pcfg = zdbsp_ProcessConfig {
				flags: zdbsp_processflags_default()
					| zdbsp_ProcessFlags_ZDBSP_PROCF_BUILDGLNODES
					| zdbsp_ProcessFlags_ZDBSP_PROCF_V5GL,
				reject_mode: zdbsp_rejectmode_default(),
				blockmap_mode: zdbsp_blockmapmode_default(),
			};

			let reader = zdbsp_wadreader_new(bytes.as_ptr());
			let p = zdbsp_processor_new(reader, std::ptr::addr_of!(pcfg));
			zdbsp_processor_run(p, std::ptr::null());

			zdbsp_processor_nodesx_v5_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodexo_callback),
			);

			zdbsp_processor_segsglx_v5_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segglx_v5_callback),
			);

			zdbsp_processor_ssectorsx_v5_foreach(
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
			// by a modified ZDBSP CLI that fills in padding bytes of subsector
			// records with zeroes.

			assert_eq!(
				format!("{:#?}", md5::compute(hash_in.subsectors.as_slice())),
				"3bc19dc80a5cbe4f704cb696853bc831"
			);

			zdbsp_processor_destroy(p);
			zdbsp_wadreader_destroy(reader);
		}
	}

	#[test]
	fn udmf_smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/udmf.wad");
		let bytes = std::fs::read(&path).unwrap();
		let mut hash_in = HashInput::default();

		unsafe {
			let reader = zdbsp_wadreader_new(bytes.as_ptr());
			let p = zdbsp_processor_new(reader, std::ptr::null());
			zdbsp_processor_run(p, std::ptr::null());

			for b in (zdbsp_processor_vertsorig_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			for b in (zdbsp_processor_vertsnew_count(p) as u32).to_le_bytes() {
				hash_in.verts.push(b);
			}

			zdbsp_processor_vertsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(vertx_callback),
			);

			for b in (zdbsp_processor_ssectorsgl_count(p) as u32).to_le_bytes() {
				hash_in.subsectors.push(b);
			}

			zdbsp_processor_ssectorsx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(ssectorx_callback),
			);

			for b in (zdbsp_processor_segsglx_count(p) as u32).to_le_bytes() {
				hash_in.segs.push(b);
			}

			zdbsp_processor_segsglx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(segglx_callback),
			);

			for b in (zdbsp_processor_nodesgl_count(p) as u32).to_le_bytes() {
				hash_in.nodes.push(b);
			}

			zdbsp_processor_nodesx_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(nodex_callback),
			);

			let mut all_bytes = vec![b'X', b'G', b'L', b'N'];

			all_bytes.append(&mut hash_in.verts);
			all_bytes.append(&mut hash_in.subsectors);
			all_bytes.append(&mut hash_in.segs);
			all_bytes.append(&mut hash_in.nodes);

			let checksum = format!("{:#?}", md5::compute(all_bytes.as_slice()));
			assert_eq!(checksum, "39ed77ca24155506b2455a887243c3ef");

			zdbsp_processor_destroy(p);
			zdbsp_wadreader_destroy(reader);
		}
	}

	#[derive(Default)]
	struct HashInput {
		segs: Vec<u8>,
		subsectors: Vec<u8>,
		nodes: Vec<u8>,
		verts: Vec<u8>,
	}

	unsafe extern "C" fn node_callback(ctx: *mut c_void, ptr: *const zdbsp_NodeRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_NodeRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn nodex_callback(ctx: *mut c_void, ptr: *const zdbsp_NodeEx) {
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

	unsafe extern "C" fn nodexo_callback(ctx: *mut c_void, ptr: *const zdbsp_NodeExO) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_NodeExO>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn seg_callback(ctx: *mut c_void, ptr: *const zdbsp_SegRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SegRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn ssector_callback(ctx: *mut c_void, ptr: *const zdbsp_SubsectorRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SubsectorRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn ssectorx_callback(ctx: *mut c_void, ptr: *const zdbsp_SubsectorEx) {
		const RECORD_SIZE: usize = std::mem::size_of::<u32>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r.num_lines);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn ssectorx_v5_callback(ctx: *mut c_void, ptr: *const zdbsp_SubsectorEx) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SubsectorEx>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn seggl_callback(ctx: *mut c_void, ptr: *const zdbsp_SegGl) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SegGl>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn segglx_callback(ctx: *mut c_void, ptr: *const zdbsp_SegGlEx) {
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

	unsafe extern "C" fn segglx_v5_callback(ctx: *mut c_void, ptr: *const zdbsp_SegGlEx) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SegGlEx>();
		let hash_in = ctx.cast::<HashInput>();
		let r = std::ptr::read(ptr);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(r);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn vertx_callback(ctx: *mut c_void, ptr: *const zdbsp_VertexEx) {
		let hash_in = ctx.cast::<HashInput>();

		for b in (*ptr).x.to_le_bytes() {
			(*hash_in).verts.push(b);
		}

		for b in (*ptr).y.to_le_bytes() {
			(*hash_in).verts.push(b);
		}
	}
}
