//! # VileTech Command Line Interface

use clap::Parser;
use indoc::printdoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Clap::parse();

	if args.version {
		println!("{}", viletech::short_version_string());
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTech Command Line Interface - Copyright (C) 2022-2023 - The Rat Circus

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation."
		};

		return Ok(());
	}

	Ok(())
}

#[derive(Debug, clap::Parser)]
struct Clap {
	/// Prints the CLI and engine versions and then exits.
	#[arg(short = 'V', long = "version")]
	version: bool,
	/// Prints license information and then exits.
	#[arg(short = 'A', long = "about")]
	about: bool,
	/// Sets the number of threads used by the global thread pool.
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	threads: Option<usize>,
}

#[must_use]
fn version_string() -> String {
	format!(
		"VileTech Command Line Interface {}",
		env!("CARGO_PKG_VERSION")
	)
}
