//! # zacs-sys

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[test]
fn smoke() {
	unsafe {
		let ctr = zacs_container_new();
		zacs_container_destroy(ctr);
	}
}
