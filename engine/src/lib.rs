//! Impure is a pet project, meant as an experiment in creating a modern,
//! feature-oriented Doom source port in Rust which can successfully interpret
//! user-generated content for GZDoom and the Eternity Engine with zero end-user
//! overhead and minimal runtime overhead.

/*
Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::{
	fs, io,
	path::{Path, PathBuf},
};

use console::ConsoleWriter;
use utils::exe_dir;

pub mod console;
pub mod data;
#[allow(dead_code)]
pub mod ecs;
#[allow(dead_code)]
pub mod file_browser;
#[allow(dead_code)]
pub mod frontend;
#[allow(dead_code)]
pub mod game;
pub mod gfx;
#[allow(dead_code)]
pub mod level;
pub mod lua;
#[allow(dead_code)]
pub mod rng;
#[allow(dead_code)]
pub mod sim;
pub mod utils;
pub mod vfs;

pub fn full_version_string() -> String {
	format!(
		"Impure engine version: {}.{}.{} (commit {}). Compiled on: {}",
		env!("CARGO_PKG_VERSION_MAJOR"),
		env!("CARGO_PKG_VERSION_MINOR"),
		env!("CARGO_PKG_VERSION_PATCH"),
		env!("GIT_HASH"),
		env!("COMPILE_DATETIME")
	)
}

/// Prepares the fern logging backend.
pub fn log_init(
	sender: crossbeam::channel::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
	let exe_dir = exe_dir();

	let colors = fern::colors::ColoredLevelConfig::new()
		.info(fern::colors::Color::Green)
		.warn(fern::colors::Color::Yellow)
		.error(fern::colors::Color::Red)
		.debug(fern::colors::Color::Cyan)
		.trace(fern::colors::Color::Magenta);

	let fpath: PathBuf = [&exe_dir, Path::new("impure.log")].iter().collect();

	if fpath.exists() {
		let oldpath: PathBuf = [&exe_dir, Path::new("impure.log.old")].iter().collect();

		match fs::rename(&fpath, oldpath) {
			Ok(()) => {}
			Err(err) => {
				eprintln!("Failed to rotate previous log file: {}", err);
			}
		};
	}

	let file_cfg = fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
				record.target(),
				record.level(),
				message
			))
		})
		.chain(
			fs::OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(fpath)?,
		);

	// Stdout logging has console colouring and less date-time elaboration
	let stdout_cfg = fern::Dispatch::new()
		.format(move |out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%H:%M:%S]"),
				record.target(),
				colors.color(record.level()),
				message
			))
		})
		.chain(io::stdout());

	let console_cfg = fern::Dispatch::new()
		.format(move |out, message, record| {
			out.finish(format_args!("[{}] {}", record.level(), message))
		})
		.chain(Box::new(ConsoleWriter::new(sender)) as Box<dyn io::Write + Send>);

	let logres = fern::Dispatch::new()
		.level(log::LevelFilter::Trace)
		.level_for("naga", log::LevelFilter::Warn)
		.level_for("wgpu_hal", log::LevelFilter::Error)
		.level_for("wgpu_core", log::LevelFilter::Error)
		.chain(console_cfg)
		.chain(file_cfg)
		.chain(stdout_cfg)
		.apply();

	if let Err(err) = logres {
		return Err(Box::new(err));
	}

	Ok(())
}

pub mod depends {
	pub extern crate chrono;
	pub extern crate crossbeam;
	pub extern crate kira;
	pub extern crate log;
	pub extern crate mlua;
	pub extern crate nanorand;
	pub extern crate parking_lot;
	pub extern crate shipyard;
	pub extern crate wgpu;
	pub extern crate winit;
}
