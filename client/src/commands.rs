//! Console command callbacks and the client's console "frontend" details.

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
	env,
	path::{Path, PathBuf},
};

use impure::{
	audio,
	console::MessageKind,
	terminal::{self, CommandArgs},
	utils::path::get_user_dir,
	vfs::ImpureVfs,
};
use indoc::formatdoc;
use kira::sound::static_sound::StaticSoundSettings;
use log::{error, info};

use crate::core::ClientCore;

pub enum Request {
	None,
	Exit,
	Callback(Box<dyn Fn(&mut ClientCore)>),
}

bitflags::bitflags! {
	pub struct CommandFlags: u8 {
		/// This command is enabled when entering the frontend
		/// and disabled when leaving it.
		const FRONTEND = 1 << 0;
		/// This command is enabled when entering the title scene
		/// and disabled when leaving it.
		const TITLE = 1 << 1;
		/// This command is enabled when entering any menu
		/// and disabled when the menu stack is cleared.
		const MENU = 1 << 2;
		/// This command is enabled when starting a playsim
		/// and disabled when it ends.
		const SIM = 1 << 3;
		/// This command is enabled when starting a lockstep
		/// network play game and disabled when it ends.
		const LOCKSTEP = 1 << 4;
		/// This command is enabled when starting an authoritative
		/// network play game and disabled when it ends.
		const AUTHORITATIVE = 1 << 5;
		/// This command is enabled when starting any network play game
		/// and disabled when it ends.
		const NETPLAY = Self::LOCKSTEP.bits | Self::AUTHORITATIVE.bits;
	}
}

pub struct Command {
	pub flags: CommandFlags,
	pub func: fn(args: terminal::CommandArgs) -> Request,
}

impl terminal::Command for Command {
	type Output = Request;

	fn call(&self, args: terminal::CommandArgs) -> Self::Output {
		(self.func)(args)
	}
}

/// Creates a console alias from a contiguous string that expands into another
/// string (whose contents can be anything, even if non-contiguous).
pub fn ccmd_alias(args: CommandArgs) -> Request {
	if args.name_only() || args.help_requested() {
		return req_console_write_help(formatdoc! {"
Define an alias, or inspect existing ones.

Usage: {} [alias] [string]

If no alias is provided, all aliases are listed. If no string is provided,
the alias' associated string is expanded into the output, if that alias exists.",
			args.command_name()
		});
	}

	let alias = args[1].to_string();

	if args.len() == 2 {
		return req_callback(move |core| match core.console.find_alias(&alias) {
			Some(a) => {
				info!("{}", a.1);
			}
			None => {
				info!("No existing alias: {}", alias);
			}
		});
	}

	let string = args.concat(2);

	req_callback(move |core| {
		info!("Alias registered: {}\r\nExpands to: {}", alias, &string);
		core.console.register_alias(alias.clone(), string.clone());
	})
}

/// Echoes every launch argument given to the client.
pub fn ccmd_args(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints out all of the program's launch arguments.".to_string(),
		);
	}

	let mut args = env::args();

	let argv0 = match args.next() {
		Some(a) => a,
		None => {
			error!("This runtime did not receive `argv[0]`.");
			return Request::None;
		}
	};

	let mut output = argv0;

	for arg in args {
		output.push('\r');
		output.push('\n');
		output.push('\t');
		output += &arg;
	}

	info!("{}", output);

	Request::None
}

/// Clears the console's message history.
pub fn ccmd_clear(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Clears the console's message history.".to_string());
	}

	req_callback(|core| {
		core.console.clear_message_history(true, true, true);
	})
}

pub fn ccmd_exit(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Instantly closes the client.".to_string());
	}

	Request::Exit
}

/// Prints the contents of a virtual file system directory,
/// or information about a file.
pub fn ccmd_file(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the contents of a virtual file system directory, \
			or information about a file."
				.to_string(),
		);
	}

	let path = PathBuf::from(if args.name_only() { "/" } else { args[1] });

	req_callback(move |core| {
		let vfsg = core.vfs.read();
		info!("{}", vfsg.ccmd_file(path.clone()));
	})
}

/// Clears the console's history of submitted input strings.
pub fn ccmd_hclear(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Clear's the console's history of submitted input strings.".to_string(),
		);
	}

	req_callback(|core| {
		info!("Clearing history of submitted input strings.");
		core.console.clear_input_history();
	})
}

/// Prints a list of all available console commands if given no arguments.
/// If the first argument is a command's name, it's equivalent to submitting
/// `command --help`.
pub fn ccmd_help(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"If used without arguments, prints a list of all available commands.\r\n\
			Giving the name of a command as a first argument is the same as giving \
			`command --help`."
				.to_string(),
		);
	}

	if args.name_only() {
		return req_callback(|core| {
			let cap = core.console.all_commands().map(|cmd| cmd.0.len()).sum();
			let mut string = String::with_capacity(cap);

			string.push_str("All available commands:");

			for command in core.console.all_commands() {
				string.push('\r');
				string.push('\n');
				string.push_str(command.0);
			}

			core.console.write(string, MessageKind::Help);
		});
	}

	let key = args[1].to_string();

	req_callback(move |core| match core.console.find_command(&key) {
		Some(cmd) => {
			(cmd.func)(terminal::CommandArgs::new(vec![&key, "--help"]));
		}
		None => {
			info!("No command found by name: {}", key);
		}
	})
}

/// Prints the directory holding the user info directory. Also see [`get_user_dir`].
pub fn ccmd_home(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the path to the directory which holds the user info directory.".to_string(),
		);
	}

	match get_user_dir() {
		Some(p) => info!("{}", p.display()),
		None => {
			info!(
				"Home directory path is malformed, \
				or this platform is unsupported."
			);
		}
	}

	Request::None
}

/// Prints the current heap memory used by the client's Lua state.
pub fn ccmd_luamem(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the current heap memory used by the client's Lua state.".to_string(),
		);
	}

	req_callback(|core| {
		info!(
			"Lua state heap usage (bytes): {}",
			core.lua.lock().used_memory()
		);
	})
}

pub fn ccmd_mididiag(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("???".to_string());
	}

	req_callback(|_| {
		unimplemented!()
	})
}

pub fn ccmd_music(args: CommandArgs) -> Request {
	if args.help_requested() || args.name_only() {
		return req_console_write_help(formatdoc! {"
Starts playing a music track.

Usage: {cmd_name} [options] <source>

<source> can (currently only) be a virtual file system path.

Options:

	--device=<midi-device>	<midi-device> can be one of the following:
								default
								std standard
								opl
								sndsys
								timidity
								fluid fluidsynth
								gus
								wildmidi
								adl
								opn
							`default` will cause the internal MIDI system to try
							to find a fallback device, but if this option isn't
							set, `fluidsynth` will be used.

	--volume=<float>		The default volume is 1.0; the given value is clamped
							between 0.0 and 1.0.
",
			cmd_name = args.command_name(),
		});
	}

	if let Some(inval) = args.any_invalid_options(&["--device", "--volume"]) {
		return req_console_write_invalidopt(inval);
	}

	if args.no_operands() {
		return req_console_write_help(
			"No virtual file path, asset ID, or asset handle provided.".to_string(),
		);
	}

	let path_string = args.operands().next().unwrap().to_string();

	let midi_dev = if let Some(option) = args.find_option(|opt| opt.starts_with("--device")) {
		match CommandArgs::option_value(option) {
			"default" => zmusic::device::Index::Default,
			"std" | "standard" => zmusic::device::Index::Standard,
			"opl" => zmusic::device::Index::Opl,
			"sndsys" => zmusic::device::Index::Sndsys,
			"timidity" => zmusic::device::Index::TiMidity,
			"fluid" | "fluidsynth" => zmusic::device::Index::FluidSynth,
			"gus" => zmusic::device::Index::Gus,
			"wildmidi" => zmusic::device::Index::WildMidi,
			"adl" => zmusic::device::Index::Adl,
			"opn" => zmusic::device::Index::Opn,
			"" => return req_console_write_help("`--device` requires a string value.".to_string()),
			other => return req_console_write_help(format!("Unknown MIDI device: `{other}`")),
		}
	} else {
		zmusic::device::Index::FluidSynth
	};

	let volume = if let Some(option) = args.find_option(|opt| opt.starts_with("--volume")) {
		let val = match CommandArgs::option_value(option) {
			"" => return req_console_write_help("`--volume` requires a string value.".to_string()),
			v => v,
		};

		match val.parse::<f64>() {
			Ok(f) => f.clamp(0.0, 1.0),
			Err(err) => {
				return req_console_write_help(format!(
					"Failed to parse `--volume` option value: {err}"
				));
			}
		}
	} else {
		1.0
	};

	req_callback(move |core| {
		let path = Path::new(&path_string);
		let vfsg = core.vfs.read();

		let fref = match vfsg.lookup(path) {
			Some(h) => h,
			None => {
				info!("No file under virtual path: {path_string}");
				return;
			}
		};

		if !fref.is_readable() {
			info!("File can not be read (neither binary nor text): {path_string}");
			return;
		}

		let bytes = fref.read_unchecked();

		if zmusic::MidiKind::is_midi(bytes) {
			let mut audio = core.audio.borrow_mut();

			let midi = match audio.zmusic.new_song(bytes, midi_dev) {
				Ok(m) => m,
				Err(err) => {
					info!("Failed to create MIDI song from: {path_string}\r\n\tError: {err}");
					return;
				}
			};

			let midi_cfg = audio.zmusic.config_global_mut();
			midi_cfg.set_master_volume(volume as f32);
			midi_cfg.set_music_volume(volume as f32);
			midi_cfg.set_relative_volume(volume as f32);

			match audio.start_music_midi::<false>(midi, false) {
				Ok(()) => {
					info!("Playing song: {path_string}\r\n\tAt volume: {volume}");
				},
				Err(err) => {
					info!("Failed to play MIDI song from: {path_string}\r\n\tError: {err}");
				}
			};
		} else if let Ok(mut sdat) = audio::sound_from_file(fref, StaticSoundSettings::default()) {
			sdat.settings.volume = kira::Volume::Amplitude(volume);

			match core.audio.borrow_mut().start_music_wave::<false>(sdat) {
				Ok(()) => {
					info!("Playing song: {path_string}\r\n\tAt volume: {volume}");
				}
				Err(err) => {
					info!("Failed to play song: {path_string}\r\nError: {err}");
				}
			};
		} else {
			info!("Given file is neither waveform nor MIDI audio: {path_string}");
		}
	})
}

/// Starts a sound at default settings from the virtual file system.
pub fn ccmd_sound(args: CommandArgs) -> Request {
	if args.help_requested() || args.name_only() {
		return req_console_write_help(formatdoc! {"
Starts a playing a sound.

Usage: {cmd_name} <source>

<source> can (currently only) be a virtual file system path.

Options:

	--volume=<float>		The default volume is 1.0; the given value is clamped
							between 0.0 and 2.0.
",
			cmd_name = args.command_name()
		});
	}

	if let Some(inval) = args.any_invalid_options(&["--device", "--volume"]) {
		return req_console_write_invalidopt(inval);
	}

	if args.no_operands() {
		return req_console_write_help(
			"No virtual file path, asset ID, or asset handle provided.".to_string(),
		);
	}

	let path_string = args.operands().next().unwrap().to_string();

	let volume = if let Some(option) = args.find_option(|opt| opt.starts_with("--volume")) {
		let val = match CommandArgs::option_value(option) {
			"" => return req_console_write_help("`--volume` requires a string value.".to_string()),
			v => v,
		};

		match CommandArgs::option_value(val).parse::<f64>() {
			Ok(f) => f,
			Err(err) => {
				return req_console_write_help(format!(
					"Failed to parse `--volume` option value: {err}"
				));
			}
		}
	} else {
		1.0
	};

	req_callback(move |core| {
		let path = Path::new(&path_string);
		let vfsg = core.vfs.read();

		let fref = match vfsg.lookup(path) {
			Some(h) => h,
			None => {
				info!("No file under virtual path: {}", path_string);
				return;
			}
		};

		if !fref.is_readable() {
			info!("File can not be read (neither binary nor text): {path_string}");
			return;
		}

		let mut sdat = match audio::sound_from_file(fref, StaticSoundSettings::default()) {
			Ok(ssd) => ssd,
			Err(err) => {
				info!("Failed to create sound from file: {}", err);
				return;
			}
		};

		sdat.settings.volume = kira::Volume::Amplitude(volume);

		match core.audio.borrow_mut().start_sound_global(sdat) {
			Ok(()) => {
				info!("Playing sound: {}", path_string);
			}
			Err(err) => {
				info!("Failed to play sound: {}", err);
			}
		};
	})
}

/// Prints the length of the time the engine has been running.
pub fn ccmd_uptime(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the length of the time the engine has been running.".to_string(),
		);
	}

	req_callback(|core| {
		info!("{}", impure::uptime_string(core.start_time));
	})
}

/// Prints information about the graphics device and WGPU backend.
pub fn ccmd_wgpudiag(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints information about the graphics device and WGPU backend.".to_string(),
		);
	}

	Request::Callback(Box::new(|core| {
		info!("{}", core.gfx.diag());
	}))
}

/// Prints the full version information of the engine and client.
pub fn ccmd_version(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the full version information of the engine and client.".to_string(),
		);
	}

	info!("{}", impure::full_version_string(&super::version_string()));
	Request::None
}

/// Prints information about the state of the virtual file system.
pub fn ccmd_vfsdiag(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints information about the state of the virtual file system.".to_string(),
		);
	}

	req_callback(|core| {
		let vfs = core.vfs.read();
		let diag = vfs.diag();
		info!(
			"Virtual file system diagnostics:\r\n\t{} {}\r\n\t{} {}\r\n\t{} {} kB",
			"Mounted objects:",
			diag.mount_count,
			"Total entries:",
			diag.num_entries,
			"Total memory usage:",
			diag.mem_usage / 1000
		);
	})
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn req_console_write_invalidopt(opt: &str) -> Request {
	let msg = format!("Unknown option: `{opt}`");

	Request::Callback(Box::new(move |core| {
		core.console.write(msg.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_console_write_help(message: String) -> Request {
	Request::Callback(Box::new(move |core| {
		core.console.write(message.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_callback<F: 'static + Fn(&mut ClientCore)>(callback: F) -> Request {
	Request::Callback(Box::new(callback))
}
