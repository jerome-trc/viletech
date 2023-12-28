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

	#[derive(Default)]
	struct HashInput {
		segs: Vec<u8>,
		subsectors: Vec<u8>,
		nodes: Vec<u8>,
	}

	unsafe extern "C" fn node_callback(ctx: *mut c_void, node: *const zdbsp_NodeRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_NodeRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(node);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).nodes.push(b);
		}
	}

	unsafe extern "C" fn seg_callback(ctx: *mut c_void, seg: *const zdbsp_SegRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SegRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(seg);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

	unsafe extern "C" fn subsector_callback(ctx: *mut c_void, seg: *const zdbsp_SubsectorRaw) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SubsectorRaw>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(seg);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).subsectors.push(b);
		}
	}

	unsafe extern "C" fn seggl_callback(ctx: *mut c_void, seg: *const zdbsp_SegGl) {
		const RECORD_SIZE: usize = std::mem::size_of::<zdbsp_SegGl>();
		let hash_in = ctx.cast::<HashInput>();
		let n = std::ptr::read(seg);
		let bytes = std::mem::transmute::<_, [u8; RECORD_SIZE]>(n);

		for b in bytes {
			(*hash_in).segs.push(b);
		}
	}

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
				Some(subsector_callback),
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

			zdbsp_processor_glnodes_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(node_callback),
			);

			zdbsp_processor_glsegs_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(seggl_callback),
			);

			zdbsp_processor_glssectors_foreach(
				p,
				std::ptr::addr_of_mut!(hash_in).cast(),
				Some(subsector_callback),
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
	#[ignore] // Until UDMF output API is complete.
	fn udmf_smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/udmf.wad");
		let bytes = std::fs::read(&path).unwrap();
		let mut out_bytes: Vec<u8> = vec![b'X', b'G', b'L', b'N'];

		unsafe {
			let reader = zdbsp_wadreader_new(bytes.as_ptr());
			let p = zdbsp_processor_new(reader, std::ptr::null());
			zdbsp_processor_run(p, std::ptr::null());

			zdbsp_processor_glnodes_foreach(
				p,
				std::ptr::addr_of_mut!(out_bytes).cast(),
				Some(node_callback),
			);

			let checksum = format!("{:#?}", md5::compute(out_bytes.as_slice()));
			assert_eq!(checksum, "39ed77ca24155506b2455a887243c3ef");

			zdbsp_processor_destroy(p);
			zdbsp_wadreader_destroy(reader);
		}
	}
}
