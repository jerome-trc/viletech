//! # VileTech Command Line Interface

use std::path::PathBuf;

use clap::Parser;
use indoc::printdoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = LaunchArgs::parse();

	if args.version {
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTools - Copyright (C) 2022-2023 - jerome-trc

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation.
"
		};

		return Ok(());
	}

	match args.subcommand {
		Commands::Midify {
			input,
			output,
			force,
		} => {
			if !input.exists() {
				eprintln!("File does not exist: `{}`", input.display());

				return Err(Box::<std::io::Error>::new(
					std::io::ErrorKind::NotFound.into(),
				));
			}

			if output.exists() && !force {
				eprintln!("A file already exists at: `{}`", output.display());
				eprintln!("Use the `--force` flag to overwrite it.");

				return Err(Box::<std::io::Error>::new(
					std::io::ErrorKind::AlreadyExists.into(),
				));
			}

			let start_time = std::time::Instant::now();

			let bytes = match std::fs::read(&input) {
				Ok(b) => b,
				Err(err) => {
					eprintln!("Failed to read file: `{}`", input.display());
					eprintln!("Details: {err}");
					return Err(Box::new(err));
				}
			};

			let smf = match midi::mus::to_midi(&bytes) {
				Ok(s) => s,
				Err(err) => {
					eprintln!("Failed to convert MUS file: `{}`", input.display());
					eprintln!("Details: {err}");
					return Err(Box::new(err));
				}
			};

			if let Err(err) = smf.save(&output) {
				eprintln!("Failed to save MIDI file: `{}`", output.display());
				eprintln!("Details: {err}");
				return Err(Box::new(err));
			}

			printdoc! {"
Converted MUS file: `{i}`
	to MIDI: `{o}`
	in: {t} ms.
",
			i = input.display(),
			o = output.display(),
			t = start_time.elapsed().as_millis()
			};

			Ok(())
		}
	}
}

#[derive(Debug, clap::Parser)]
struct LaunchArgs {
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
	#[command(subcommand)]
	subcommand: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
	/// Converts a MUS file to a MIDI file.
	Midify {
		/// A MUS file to convert.
		#[arg(short, long)]
		input: PathBuf,
		/// The path to write the converted MIDI file to.
		#[arg(short, long)]
		output: PathBuf,
		/// If set, any file at `output` is allowed to overwritten.
		#[arg(short, long)]
		force: bool,
	},
}

#[must_use]
fn version_string() -> String {
	format!("VileTools {}", env!("CARGO_PKG_VERSION"))
}
