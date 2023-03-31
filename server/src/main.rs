//! VileTech Dedicated Server

// TODO: Remove
#![allow(dead_code)]
#![allow(unused)]

mod commands;

use std::{error::Error, time::Instant};

use clap::Parser;
use indoc::printdoc;
use vile::terminal::Terminal;

use commands::Command;

#[must_use]
pub fn version_string() -> String {
	format!("VileTech Server {}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
pub struct ServerCore {
	/// (RAT) In my experience, a runtime log is much more informative if it
	/// states the duration for which the program executed.
	start_time: Instant,
	terminal: Terminal<Command>,
}

#[derive(Parser, Debug)]
struct Clap {
	#[arg(short = 'V', long = "version")]
	version: bool,
	#[arg(short = 'A', long = "about")]
	about: bool,

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
	let start_time = Instant::now();
	let args = Clap::parse();

	if args.version {
		println!("{}", vile::short_version_string());
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTech Server - Copyright (C) 2022-2023 - ***REMOVED***

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation."
		};

		return Ok(());
	}

	vile::thread_pool_init(args.threads);

	match vile::log_init(None) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {err}");
			return Err(err);
		}
	}

	vile::log_init_diag(&version_string())?;

	// Soon!

	Ok(())
}
