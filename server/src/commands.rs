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

use std::env;

use impure::{
	terminal::{self, CommandArgs},
	utils::path::get_user_dir,
};
use log::{error, info};

use crate::ServerCore;

pub enum Request {
	None,
	Exit,
	Callback(Box<dyn Fn(&mut ServerCore)>),
}

bitflags::bitflags! {
	/// A command is enabled if it one of its active bits corresponds to the
	/// server's current "context".
	pub struct Flags : u8 {
		const LOBBY = 1 << 0;
		const SIM = 1 << 1;
	}
}

pub struct Command {
	pub flags: Flags,
	pub func: fn(args: CommandArgs) -> Request,
}

impl terminal::Command for Command {
	type Output = Request;

	fn call(&self, args: CommandArgs) -> Self::Output {
		(self.func)(args)
	}
}

pub fn cmd_alias(args: CommandArgs) -> Request {
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

	let alias = args[1].to_string();

	if args.name_only() || args.help_requested() {
		help(args[0]);
		return Request::None;
	}

	if args.len() == 2 {
		return req_callback(move |core| match core.terminal.find_alias(&alias) {
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
		core.terminal.register_alias(alias.clone(), string.clone());
	})
}

pub fn cmd_args(args: CommandArgs) -> Request {
	if args.help_requested() {
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

pub fn cmd_help(args: CommandArgs) -> Request {
	if args.help_requested() {
		info!(
			"If used without arguments, prints a list of all available commands.
			Giving the name of a command as a first argument is the same as giving
			`command --help`."
		);
		return Request::None;
	}

	if args.name_only() {
		return req_callback(|core| {
			let mut string = "All available commands:".to_string();

			for command in core.terminal.all_commands() {
				string.push('\r');
				string.push('\n');
				string.push_str(command.0);
			}

			info!("{}", string);
		});
	}

	let key = args[1].to_string();

	req_callback(move |core| match core.terminal.find_command(&key) {
		Some(cmd) => {
			(cmd.func)(terminal::CommandArgs::new(vec![&key, "--help"]));
		}
		None => {
			info!("No command found by name: {}", key);
		}
	})
}

pub fn cmd_home(args: CommandArgs) -> Request {
	if args.help_requested() {
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

pub fn cmd_quit(args: CommandArgs) -> Request {
	if args.help_requested() {
		info!("Instantly closes the application.");
		return Request::None;
	}

	Request::Exit
}

pub fn cmd_uptime(args: CommandArgs) -> Request {
	if args.help_requested() {
		info!("Prints the current cumulative uptime of the application.");
		return Request::None;
	}

	req_callback(|core| {
		info!("{}", impure::uptime_string(core.start_time));
	})
}

pub fn cmd_version(args: CommandArgs) -> Request {
	if args.help_requested() {
		info!("Prints the engine version.");
		return Request::None;
	}

	info!("{}", impure::full_version_string(&super::version_string()));
	Request::None
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn req_callback<F: 'static + Fn(&mut ServerCore)>(callback: F) -> Request {
	Request::Callback(Box::new(callback))
}
