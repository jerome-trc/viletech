//! VileTech Dedicated Server

mod commands;

use std::{error::Error, time::Instant};

use bevy::prelude::*;
use clap::Parser;
use indoc::printdoc;
use viletech::{terminal::Terminal, util::duration_to_hhmmss};

use commands::Command;

#[must_use]
pub fn version_string() -> String {
	format!("VileTech Server {}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
pub struct ServerCore {
	pub start_time: Instant,
	pub terminal: Terminal<Command>,
}

#[derive(clap::Parser, Debug)]
#[command(name = "VileTech Server")]
#[command(version)]
#[command(about = "Dedicated server for the VileTech Engine")]
#[command(long_about = "
VileTech Server - Copyright (C) 2022-2023 - jerome-trc

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation.")]
struct LaunchArgs {
	/// Version info for both the server and engine.
	///
	/// Same as `--version` along with the version, Git commit SHA, and compile
	/// timestamp of the `viletech` "engine" library.
	#[arg(long)]
	version_full: bool,
	/// Sets the number of threads used by the global thread pool
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	threads: Option<usize>,

	/// If not set, this defaults to 64.
	#[clap(long, value_parser, default_value_t = 64)]
	max_clients: usize,
	/// Can be empty.
	#[clap(long, value_parser, default_value = "")]
	password: String,
	/// If not set, this defaults to 6666.
	#[clap(long, value_parser, default_value_t = 6666)]
	port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
	viletech::START_TIME.set(Instant::now()).unwrap();
	let args = LaunchArgs::parse();

	if args.version_full {
		let s_vers = env!("CARGO_PKG_VERSION");
		let [e_vers, commit, comp_datetime] = viletech::version_info();

		printdoc! {"
VileTech Server {s_vers}
{e_vers}
{commit}
{comp_datetime}
"};

		return Ok(());
	}

	viletech::thread_pool_init(args.threads);
	viletech::log::init_diag(&version_string())?;

	// (RAT) In my experience, a runtime log is much more informative if it
	// states the duration for which the program executed.
	let uptime = viletech::START_TIME.get().unwrap().elapsed();
	let (hh, mm, ss) = duration_to_hhmmss(uptime);
	info!("Uptime: {hh:02}:{mm:02}:{ss:02}");

	Ok(())
}
