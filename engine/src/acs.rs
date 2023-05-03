//! [Action Code Script] execution environment and supporting infrastructure.
//!
//! Assume all code within originates from GZDoom-original source.
//!
//! [Action Code Script]: https://doomwiki.org/wiki/ACS

#![allow(dead_code)] // TODO: Disallow

mod constants;
mod detail;
mod funcs;
mod pcodes;
mod script;
mod strpool;

/// ACS demands sweeping access to information at several levels of the engine.
/// This gets constructed per-tick from the playsim loop and passed down to run
/// scripts with.
#[derive(Debug)]
pub struct Context {}

#[derive(Debug)]
pub struct Controller {}

impl Controller {
	fn tick(&self) {
		todo!()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
	Old,
	Enhanced,
	LittleEnhanced,
	Unknown,
}

type Array = Vec<i32>;
