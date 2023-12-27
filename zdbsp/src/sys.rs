//! Automatically-generated FFI bindings to zdbsp-rs' C header.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Tests for verifying the correctness of changes made to upstream ZDBSP.

#[cfg(test)]
use std::{ffi::c_void, path::Path};

#[test]
fn smoke() {
	unsafe extern "C" fn node_callback(ctx: *mut c_void, node: *const zdbsp_NodeRaw) {
		const NODE_SIZE: usize = std::mem::size_of::<zdbsp_NodeRaw>();
		let out_bytes = ctx.cast::<Vec<u8>>();
		let n = std::ptr::read(node);
		let node_bytes = std::mem::transmute::<_, [u8; NODE_SIZE]>(n);

		for b in node_bytes {
			(*out_bytes).push(b);
		}
	}

	let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
	let bytes = std::fs::read(&path).unwrap();
	let mut out_bytes: Vec<u8> = vec![];

	unsafe {
		let reader = zdbsp_wadreader_new(bytes.as_ptr());
		let p = zdbsp_processor_new(reader, std::ptr::null());
		zdbsp_processor_run(p, std::ptr::null());

		zdbsp_processor_nodes_foreach(
			p,
			std::ptr::addr_of_mut!(out_bytes).cast(),
			Some(node_callback),
		);

		let checksum = format!("{:#?}", md5::compute(out_bytes.as_slice()));
		assert_eq!(checksum, "375e670aef63eddb364b41b40f19ee02");

		zdbsp_processor_destroy(p);
		zdbsp_wadreader_destroy(reader);
	}
}

#[test]
fn smoke_udmf() {
	unsafe extern "C" fn nodex_callback(ctx: *mut c_void, node: *const zdbsp_NodeEx) {
		const NODE_SIZE: usize = std::mem::size_of::<zdbsp_NodeEx>();
		let out_bytes = ctx.cast::<Vec<u8>>();
		let n = std::ptr::read(node);
		let node_bytes = std::mem::transmute::<_, [u8; NODE_SIZE]>(n);

		for b in node_bytes {
			(*out_bytes).push(b);
		}
	}

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
			Some(nodex_callback),
		);

		let checksum = format!("{:#?}", md5::compute(out_bytes.as_slice()));
		assert_eq!(checksum, "39ed77ca24155506b2455a887243c3ef");

		zdbsp_processor_destroy(p);
		zdbsp_wadreader_destroy(reader);
	}
}
