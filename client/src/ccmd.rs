//! Console command callbacks and the client's console "frontend" details.

use std::env;

use indoc::formatdoc;
use log::{error, info};
use vile::{
	console::MessageKind,
	terminal::{self, CommandArgs},
	utils::duration_to_hhmmss,
};

use crate::core::ClientCore;

pub enum Request {
	None,
	Exit,
	Callback(Box<dyn Fn(&mut ClientCore)>),
}

impl std::fmt::Debug for Request {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::Exit => write!(f, "Exit"),
			Self::Callback(_) => f.debug_tuple("Callback").finish(),
		}
	}
}

bitflags::bitflags! {
	pub struct Flags: u8 {
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
	pub flags: Flags,
	pub func: fn(args: terminal::CommandArgs) -> Request,
}

impl terminal::Command for Command {
	type Output = Request;

	fn call(&self, args: terminal::CommandArgs) -> Self::Output {
		(self.func)(args)
	}
}

impl std::fmt::Debug for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Command")
			.field("flags", &self.flags)
			.finish()
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
		return req_console_write_help("Prints out all of the program's launch arguments.");
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
		return req_console_write_help("Clears the console's message history.");
	}

	req_callback(|core| {
		core.console.clear_message_history(true, true, true);
	})
}

pub fn ccmd_exit(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Instantly closes the client.");
	}

	Request::Exit
}

/// Clears the console's history of submitted input strings.
pub fn ccmd_hclear(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Clear's the console's history of submitted input strings.");
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
			`command --help`.",
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

/// Prints the length of the time the engine has been running.
pub fn ccmd_uptime(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the length of the time the engine has been running.",
		);
	}

	req_callback(|core| {
		let uptime = core.start_time.elapsed();
		let (hh, mm, ss) = duration_to_hhmmss(uptime);
		info!("Uptime: {hh:02}:{mm:02}:{ss:02}");
	})
}

/// Prints information about the graphics device and WGPU backend.
pub fn ccmd_wgpudiag(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints information about the graphics device and WGPU backend.",
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
			"Prints the full version information of the engine and client.",
		);
	}

	info!("{}", vile::full_version_string(&super::version_string()));
	Request::None
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
#[allow(unused)]
fn req_console_write_invalidopt(opt: &str) -> Request {
	let msg = format!("Unknown option: `{opt}`");

	Request::Callback(Box::new(move |core| {
		core.console.write(msg.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_console_write_help(message: impl Into<String>) -> Request {
	let message = message.into();

	Request::Callback(Box::new(move |core| {
		core.console.write(message.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_callback<F: 'static + Fn(&mut ClientCore)>(callback: F) -> Request {
	Request::Callback(Box::new(callback))
}