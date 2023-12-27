//! # zdbsp-rs
//!
//! A safe Rust wrapper around a fork of [ZDBSP], the BSP node tree builder used
//! by ZDoom-family source ports of the id Tech 1 game engine.
//!
//! [ZDBSP]: https://zdoom.org/wiki/ZDBSP

pub mod sys {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]

	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(test)]
mod test {
	use std::path::Path;

	use super::*;

	#[test]
	fn smoke() {
		let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/freedoom2/map01.wad");
		let bytes = std::fs::read(&path).unwrap();

		unsafe extern "C" fn node_callback(
			ctx: *mut std::ffi::c_void,
			node: *const sys::zdbsp_MapNode,
		) {
			let visited = ctx.cast::<bool>();

			if *visited {
				return;
			}

			*visited = true;

			assert_eq!([(*node).x, (*node).y], [1704, -384]);
			assert_eq!([(*node).dx, (*node).dy], [32, 0]);
		}

		unsafe extern "C" fn nodex_callback(
			ctx: *mut std::ffi::c_void,
			node: *const sys::zdbsp_MapNodeEx,
		) {
			let visited = ctx.cast::<bool>();

			if *visited {
				return;
			}

			*visited = true;

			assert_eq!([(*node).x, (*node).y], [111673344, -25165824]);
		}

		unsafe {
			let reader = sys::zdbsp_wadreader_new(bytes.as_ptr());
			let p = sys::zdbsp_processor_new(reader, std::ptr::null());
			sys::zdbsp_processor_run(p, std::ptr::null());

			assert_eq!(sys::zdbsp_processor_nodesx_count(p), 616);

			let mut visited_node = false;

			sys::zdbsp_processor_nodes_foreach(
				p,
				std::ptr::addr_of_mut!(visited_node).cast(),
				Some(node_callback),
			);

			let mut visited_nodex = false;

			sys::zdbsp_processor_nodesx_foreach(
				p,
				std::ptr::addr_of_mut!(visited_nodex).cast(),
				Some(nodex_callback),
			);

			sys::zdbsp_processor_destroy(p);
			sys::zdbsp_wadreader_destroy(reader);
		}
	}

	#[test]
	fn smoke_udmf() {
		unsafe extern "C" fn nodex_callback(
			ctx: *mut std::ffi::c_void,
			node: *const sys::zdbsp_MapNodeEx,
		) {
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
			let reader = sys::zdbsp_wadreader_new(bytes.as_ptr());
			let p = sys::zdbsp_processor_new(reader, std::ptr::null());
			sys::zdbsp_processor_run(p, std::ptr::null());

			let mut visited_nodex = false;

			sys::zdbsp_processor_glnodes_foreach(
				p,
				std::ptr::addr_of_mut!(visited_nodex).cast(),
				Some(nodex_callback),
			);

			sys::zdbsp_processor_destroy(p);
			sys::zdbsp_wadreader_destroy(reader);
		}
	}
}
