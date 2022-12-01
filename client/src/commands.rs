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

use std::{env, path::PathBuf};

use impure::{
	console::MessageKind,
	depends::{
		bitflags,
		log::{error, info},
	},
	terminal::{self, CommandArgs},
	utils::path::get_user_dir,
};

use crate::core::ClientCore;

pub enum Request {
	None,
	Callback(fn(&mut ClientCore)),
	Exit,
	ConsoleWrite(String, MessageKind),
	CommandHelp(String),
	CreateAlias(String, String),
	EchoAlias(String),
	File(PathBuf),
	Sound(String),
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

pub fn ccmd_alias(args: CommandArgs) -> Request {
	if args.id_only() || args.help() {
		return Request::ConsoleWrite(
			format!(
				"Usage: {} [alias] [string]
If no alias is provided, all aliases are listed. If no string is provided,
the alias' associated string is expanded into the output, if that alias exists.",
				args[0]
			),
			MessageKind::Help,
		);
	}

	if args.len() == 2 {
		return Request::EchoAlias(args[1].to_string());
	}

	Request::CreateAlias(args[1].to_string(), CommandArgs::concat(&args[2..]))
}

pub fn ccmd_args(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints out all of the program's launch arguments.".to_string(),
			MessageKind::Help,
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

pub fn ccmd_clear(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Clears the console's message log.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
		core.console.clear_log();
	})
}

pub fn ccmd_exit(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Instantly closes the client.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Exit
}

pub fn ccmd_file(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints the contents of a virtual file system directory, \
or information about a file."
				.to_string(),
			MessageKind::Help,
		);
	}

	Request::File(PathBuf::from(if args.id_only() { "/" } else { args[1] }))
}

pub fn ccmd_hclear(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Clear's the console's history of submitted input strings.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
		info!("Clearing history of submitted input strings.");
		core.console.clear_input_history();
	})
}

pub fn ccmd_help(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"If used without arguments, prints a list of all available commands.
Giving the name of a command as a first argument is the same as giving
`command --help`."
				.to_string(),
			MessageKind::Help,
		);
	}

	if args.id_only() {
		return Request::Callback(|core| {
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

	Request::CommandHelp(args[1].to_string())
}

pub fn ccmd_home(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints the directory which holds the user info directory.".to_string(),
			MessageKind::Help,
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

pub fn ccmd_luamem(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints the current heap memory used by the client's Lua state.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
		info!(
			"Lua state heap usage (bytes): {}",
			core.lua.lock().used_memory()
		);
	})
}

pub fn ccmd_sound(args: CommandArgs) -> Request {
	if args.help() || args.is_empty() {
		return Request::ConsoleWrite(
			format!(
				"Starts a sound at default settings from the virtual file system.
Usage: {} <virtual file path/asset number/asset ID>",
				args[0]
			),
			MessageKind::Help,
		);
	}

	Request::Sound(args[1].to_string())
}

pub fn ccmd_uptime(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints the length of the time the engine has been running.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
		info!("{}", impure::uptime_string(core.start_time));
	})
}

pub fn ccmd_wgpudiag(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints information about the graphics device and WGPU backend.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
		info!("{}", core.gfx.diag());
	})
}

pub fn ccmd_version(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite("Prints the engine version.".to_string(), MessageKind::Help);
	}

	info!("{}", impure::full_version_string());
	Request::None
}

pub fn ccmd_vfsdiag(args: CommandArgs) -> Request {
	if args.help() {
		return Request::ConsoleWrite(
			"Prints information about the state of the virtual file system.".to_string(),
			MessageKind::Help,
		);
	}

	Request::Callback(|core| {
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
