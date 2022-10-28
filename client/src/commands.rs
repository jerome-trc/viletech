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
	fn help(cmd_key: &str) {
		info!(
			"Usage: {} [alias] [string]
			If no alias is provided, all aliases are listed.
			If no string is provided, \
			the alias' associated string is expanded into the output, \
			if that alias exists.",
			cmd_key
		);
	}

	if args.id_only() || args.help() {
		help(args[0]);
		return Request::None;
	}

	if args.len() == 2 {
		return Request::EchoAlias(args[1].to_string());
	}

	Request::CreateAlias(args[1].to_string(), CommandArgs::concat(&args[2..]))
}

pub fn ccmd_args(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints out all of the program's launch arguments.");
		return Request::None;
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
		info!("Clears the console's message log.");
		return Request::None;
	}

	Request::Callback(|core| {
		core.console.clear_log();
	})
}

pub fn ccmd_exit(args: CommandArgs) -> Request {
	if args.help() {
		info!("Instantly closes the client.");
		return Request::None;
	}

	Request::Exit
}

pub fn ccmd_file(args: CommandArgs) -> Request {
	if args.help() {
		info!(
			"Prints the contents of a virtual file system directory, \
			or information about a file."
		);
		return Request::None;
	}

	Request::File(PathBuf::from(if args.id_only() { "/" } else { args[1] }))
}

pub fn ccmd_hclear(args: CommandArgs) -> Request {
	if args.help() {
		info!("Clear's the console's history of submitted input strings.");
		return Request::None;
	}

	Request::Callback(|core| {
		info!("Clearing history of submitted input strings.");
		core.console.clear_input_history();
	})
}

pub fn ccmd_help(args: CommandArgs) -> Request {
	if args.help() {
		info!(
			"If used without arguments, prints a list of all available commands.
			Giving the name of a command as a first argument is the same as giving
			`command --help`."
		);
		return Request::None;
	}

	if args.id_only() {
		return Request::Callback(|core| {
			let mut string = "All available commands:".to_string();

			for command in core.console.all_commands() {
				string.push('\r');
				string.push('\n');
				string.push_str(command.0);
			}

			info!("{}", string);
		});
	}

	Request::CommandHelp(args[1].to_string())
}

pub fn ccmd_home(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints the directory which holds the user info directory.");
		return Request::None;
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
		info!("Prints the current heap memory used by the client's Lua state.");
		return Request::None;
	}

	Request::Callback(|core| {
		info!(
			"Lua state heap usage (bytes): {}",
			core.lua.lock().used_memory()
		);
	})
}

pub fn ccmd_sound(args: CommandArgs) -> Request {
	fn help(cmd_key: &str) {
		info!(
			"Starts a sound at default settings from the virtual file s
			Usage: {} <virtual file path/asset number/asset ID>",
			cmd_key
		);
	}

	if args.help() || args.is_empty() {
		help(args[0]);
		return Request::None;
	}

	Request::Sound(args[1].to_string())
}

pub fn ccmd_uptime(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints the length of the time the engine has been running.");
		return Request::None;
	}

	Request::Callback(|core| {
		info!("{}", impure::uptime_string(core.start_time));
	})
}

pub fn ccmd_wgpudiag(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints information about the graphics device and WGPU backend.");
		return Request::None;
	}

	Request::Callback(|core| {
		info!("{}", core.gfx.diag());
	})
}

pub fn ccmd_version(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints the engine version.");
		return Request::None;
	}

	info!("{}", impure::full_version_string());
	Request::None
}

pub fn ccmd_vfsdiag(args: CommandArgs) -> Request {
	if args.help() {
		info!("Prints information about the state of the virtual file system.");
		return Request::None;
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
