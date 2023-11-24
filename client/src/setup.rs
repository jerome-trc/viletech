//! Functions for setting up the Bevy app before any meaningful execution starts.

use std::time::{Duration, Instant};

use bevy::{
	app::PluginGroupBuilder,
	log::LogPlugin,
	prelude::*,
	render::{
		settings::{RenderCreation, WgpuFeatures, WgpuSettings},
		RenderPlugin,
	},
	window::WindowMode,
	winit::{UpdateMode, WinitSettings},
};
use viletech::{
	console::{self, Console},
	crossbeam::channel::{Receiver, Sender},
	log::TracingPlugin,
};

use crate::ccmd;

#[derive(Debug, clap::Parser)]
#[command(name = "VileTech Client")]
#[command(version)]
#[command(about = "Client for the VileTech Engine")]
#[command(long_about = "
VileTech Client - Copyright (C) 2022-2023 - jerome-trc

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation.")]
pub(crate) struct LaunchArgs {
	/// Version info for both the client and engine.
	///
	/// Same as `--version` along with the version, Git commit SHA, and compile
	/// timestamp of the `viletech` "engine" library.
	#[arg(long)]
	pub(crate) version_full: bool,
	/// Sets the number of threads used by the global thread pool.
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	pub(crate) threads: Option<usize>,
	/// Sets how much logging goes to stdout, the console, and log files.
	///
	/// Possible values: ERROR, WARN, INFO, DEBUG, or TRACE.
	#[arg(short, long, default_value_t = viletech::log::Level::INFO)]
	pub(crate) verbosity: viletech::log::Level,
}

#[must_use]
pub(crate) fn default_plugins(
	start_time: Instant,
	args: &LaunchArgs,
	log_sender: Sender<console::Message>,
) -> PluginGroupBuilder {
	DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				title: "VileTech Client".to_string(),
				mode: WindowMode::Windowed,
				..Default::default()
			}),
			..default()
		})
		.set(TaskPoolPlugin {
			task_pool_options: TaskPoolOptions::with_num_threads(args.threads.unwrap_or_else(
				|| {
					std::thread::available_parallelism()
						.map(|u| u.get())
						.unwrap_or(0)
				},
			)),
		})
		.set(RenderPlugin {
			render_creation: RenderCreation::Automatic(WgpuSettings {
				features: WgpuFeatures::default() | WgpuFeatures::POLYGON_MODE_LINE,
				..default()
			}),
		})
		.disable::<LogPlugin>()
		.add_before::<TaskPoolPlugin, _>(TracingPlugin {
			start_time,
			console_sender: Some(log_sender),
			level: args.verbosity,
			filter: "wgpu=error".to_string(),
		})
}

#[must_use]
pub(crate) fn winit_settings() -> WinitSettings {
	WinitSettings {
		return_from_run: false,
		focused_mode: UpdateMode::Reactive {
			wait: Duration::from_secs_f64(1.0 / 60.0),
		},
		unfocused_mode: UpdateMode::ReactiveLowPower {
			wait: Duration::from_secs_f64(1.0 / 30.0),
		},
	}
}

#[must_use]
pub(crate) fn console(receiver: Receiver<console::Message>) -> Console<ccmd::Command> {
	let mut console = Console::new(receiver);

	console.register_command(
		"alias",
		ccmd::Command {
			func: ccmd::ccmd_alias,
		},
		true,
	);

	console.register_command(
		"args",
		ccmd::Command {
			func: ccmd::ccmd_args,
		},
		true,
	);

	console.register_command(
		"clear",
		ccmd::Command {
			func: ccmd::ccmd_clear,
		},
		true,
	);

	console.register_command(
		"exit",
		ccmd::Command {
			func: ccmd::ccmd_exit,
		},
		true,
	);

	console.register_command(
		"hclear",
		ccmd::Command {
			func: ccmd::ccmd_hclear,
		},
		true,
	);

	console.register_command(
		"help",
		ccmd::Command {
			func: ccmd::ccmd_help,
		},
		true,
	);

	console.register_command(
		"version",
		ccmd::Command {
			func: ccmd::ccmd_version,
		},
		true,
	);

	console.register_alias("quit".to_string(), "exit".to_string());

	console
}
