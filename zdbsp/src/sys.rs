#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Tests for verifying the correctness of changes made to upstream ZDBSP.

#[cfg(test)]
use std::path::Path;

#[test]
fn smoke() {
	let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
	let bytes = std::fs::read(&path).unwrap();

	unsafe extern "C" fn node_callback(ctx: *mut std::ffi::c_void, node: *const zdbsp_NodeRaw) {
		let visited = ctx.cast::<bool>();

		if *visited {
			return;
		}

		*visited = true;

		assert_eq!([(*node).x, (*node).y], [1704, -384]);
		assert_eq!([(*node).dx, (*node).dy], [32, 0]);
	}

	unsafe extern "C" fn nodex_callback(ctx: *mut std::ffi::c_void, node: *const zdbsp_NodeEx) {
		let visited = ctx.cast::<bool>();

		if *visited {
			return;
		}

		*visited = true;

		assert_eq!([(*node).x, (*node).y], [111673344, -25165824]);
	}

	unsafe {
		let reader = zdbsp_wadreader_new(bytes.as_ptr());
		let p = zdbsp_processor_new(reader, std::ptr::null());
		zdbsp_processor_run(p, std::ptr::null());

		assert_eq!(zdbsp_processor_nodesx_count(p), 616);

		let mut visited_node = false;

		zdbsp_processor_nodes_foreach(
			p,
			std::ptr::addr_of_mut!(visited_node).cast(),
			Some(node_callback),
		);

		let mut visited_nodex = false;

		zdbsp_processor_nodesx_foreach(
			p,
			std::ptr::addr_of_mut!(visited_nodex).cast(),
			Some(nodex_callback),
		);

		zdbsp_processor_destroy(p);
		zdbsp_wadreader_destroy(reader);
	}
}

#[test]
fn smoke_udmf() {
	unsafe extern "C" fn nodex_callback(ctx: *mut std::ffi::c_void, node: *const zdbsp_NodeEx) {
		let visited = ctx.cast::<bool>();

		if *visited {
			return;
		}

		*visited = true;

		assert_eq!([(*node).x, (*node).y], [0, 6291456]);
		assert_eq!([(*node).dx, (*node).dy], [12582912, 6291456]);
	}

	let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/udmf.wad");
	let bytes = std::fs::read(&path).unwrap();

	unsafe {
		let reader = zdbsp_wadreader_new(bytes.as_ptr());
		let p = zdbsp_processor_new(reader, std::ptr::null());
		zdbsp_processor_run(p, std::ptr::null());

		let mut visited_nodex = false;

		zdbsp_processor_glnodes_foreach(
			p,
			std::ptr::addr_of_mut!(visited_nodex).cast(),
			Some(nodex_callback),
		);

		zdbsp_processor_destroy(p);
		zdbsp_wadreader_destroy(reader);
	}
}
