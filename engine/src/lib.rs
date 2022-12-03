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

#[allow(dead_code)]
pub mod acs;
pub mod audio;
pub mod console;
pub mod cvarinfo;
pub mod data;
#[allow(dead_code)]
pub mod ecs;
#[allow(dead_code)]
pub mod frontend;
#[allow(dead_code)]
pub mod game;
pub mod gfx;
pub mod input;
#[allow(dead_code)]
pub mod level;
pub mod lith;
pub mod lua;
pub mod math;
#[allow(dead_code)]
pub mod rng;
#[allow(dead_code)]
pub mod sim;
pub mod terminal;
#[allow(dead_code)]
pub mod user;
pub mod utils;
pub mod vfs;
pub mod wad;
#[allow(dead_code)]
pub mod zscript;

// Type aliases

pub use level::Cluster as LevelCluster;
pub use level::Flags as LevelFlags;
pub use level::Metadata as LevelMetadata;
pub use vfs::Error as VfsError;

// Re-export transitive dependencies

pub extern crate num_derive;
pub extern crate num_traits;

pub mod depends {
	pub extern crate bitflags;
	pub extern crate bytemuck;
	pub extern crate chrono;
	pub extern crate crossbeam;
	pub extern crate dolly;
	pub extern crate fasthash;
	pub extern crate kira;
	pub extern crate log;
	pub extern crate mlua;
	pub extern crate nanorand;
	pub extern crate parking_lot;
	pub extern crate regex;
	pub extern crate renet;
	pub extern crate sha3;
	pub extern crate shipyard;
	pub extern crate wgpu;
	pub extern crate winit;
	pub extern crate zip;
}

// Symbols that don't belong in any other module

#[must_use]
pub fn short_version_string() -> String {
	format!("Impure Engine version {}.", env!("CARGO_PKG_VERSION"))
}

#[must_use]
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
	sender: Option<crossbeam::channel::Sender<console::Message>>,
) -> Result<(), Box<dyn std::error::Error>> {
	use console::Writer;
	use std::{
		fs, io,
		path::{Path, PathBuf},
	};

	let exe_dir = utils::path::exe_dir();

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

	let dispatch = fern::Dispatch::new()
		.level(log::LevelFilter::Trace)
		.level_for("naga", log::LevelFilter::Warn)
		.level_for("wgpu_hal", log::LevelFilter::Error)
		.level_for("wgpu_core", log::LevelFilter::Error)
		.level_for("symphonia_core", log::LevelFilter::Warn)
		.level_for("symphonia_format_ogg", log::LevelFilter::Warn)
		.level_for("symphonia_codec_vorbis", log::LevelFilter::Warn)
		.level_for("symphonia_bundle_mp3", log::LevelFilter::Warn)
		.chain(file_cfg)
		.chain(stdout_cfg);

	if let Some(s) = sender {
		let console_cfg = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!("[{}] {}", record.level(), message))
			})
			.chain(Box::new(Writer::new(s)) as Box<dyn io::Write + Send>);

		dispatch.chain(console_cfg).apply()
	} else {
		dispatch.apply()
	}?;

	Ok(())
}

#[must_use]
pub fn uptime_string(start_time: std::time::Instant) -> String {
	let elapsed = start_time.elapsed();
	let dur = chrono::Duration::from_std(elapsed).unwrap();
	let secs = dur.num_seconds();
	let mins = secs / 60;
	let hours = mins / 60;
	format!("Uptime: {:02}:{:02}:{:02}", hours, mins % 60, secs % 60)
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn log_cpu_info() {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn log_cpu_info() {
	let cpuid = raw_cpuid::CpuId::new();
	let mut output = String::with_capacity(512);

	if let Some(vendor) = cpuid.get_vendor_info() {
		output.push_str(&format!("\t- Vendor ID: \"{}\"\r\n", vendor.as_str()))
	} else {
		output.push_str("\t- Vendor ID: <unknown>\r\n");
	};

	if let Some(pbs) = cpuid.get_processor_brand_string() {
		output.push_str(&format!("\t- Name: \"{}\"\r\n", pbs.as_str()));
	} else {
		output.push_str("\t- Name: <unknown>\r\n");
	}

	if let Some(feats) = cpuid.get_feature_info() {
		output.push_str(&format!(
			"\t- Family ID: {} ({} base, {} extended)
\t- Model ID: {} ({} base, {} extended)
\t- Stepping ID: {}\r\n",
			feats.family_id(),
			feats.base_family_id(),
			feats.extended_family_id(),
			feats.model_id(),
			feats.base_model_id(),
			feats.extended_model_id(),
			feats.stepping_id()
		));

		output.push_str("\t- Features:");

		if feats.has_avx() {
			output.push_str(" AVX");
		}

		if feats.has_f16c() {
			output.push_str(" F16C");
		}

		if feats.has_fma() {
			output.push_str(" FMA");
		}

		if feats.has_sse() {
			output.push_str(" SSE");
		}

		if feats.has_sse2() {
			output.push_str(" SSE2");
		}

		if feats.has_sse3() {
			output.push_str(" SSE3");
		}

		if feats.has_sse41() {
			output.push_str(" SSE4.1");
		}

		if feats.has_sse42() {
			output.push_str(" SSE4.2");
		}

		if output.ends_with("Features:") {
			output.push_str(" <none>");
		}
	} else {
		output.push_str("\t- Feature/family information not found\r\n");
	}

	log::info!("CPU diagnostics: \r\n{}", output);
}
