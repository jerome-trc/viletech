//! Console command callbacks and the client's console "frontend" details.

use std::env;

use bevy::{
	app::AppExit,
	ecs::system::SystemState,
	prelude::*,
	render::{
		renderer::RenderDevice,
		settings::{WgpuFeatures, WgpuLimits},
	},
};
use indoc::formatdoc;
use viletech::{
	console::MessageKind,
	terminal::{self, CommandArgs},
	tracing::{error, info},
};

use crate::dgui::Console;

pub(crate) enum Request {
	None,
	Callback(Box<dyn 'static + Fn(&mut World) + Send + Sync>),
}

impl std::fmt::Debug for Request {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::Callback(_) => f.debug_tuple("Callback").finish(),
		}
	}
}

pub(crate) struct Command {
	pub(crate) func: fn(args: terminal::CommandArgs) -> Request,
}

impl terminal::Command for Command {
	type Output = Request;

	fn call(&self, args: terminal::CommandArgs) -> Self::Output {
		(self.func)(args)
	}
}

impl std::fmt::Debug for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Command").finish()
	}
}

/// Creates a console alias from a contiguous string that expands into another
/// string (whose contents can be anything, even if non-contiguous).
pub(crate) fn ccmd_alias(args: CommandArgs) -> Request {
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
		return req_callback(move |eworld| {
			let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
			let console = sys.get_mut(eworld);

			match console.find_alias(&alias) {
				Some(a) => {
					info!("{}", a.expanded);
				}
				None => {
					info!("No existing alias: {}", alias);
				}
			}
		});
	}

	let string = args.concat(2);

	req_callback(move |eworld| {
		info!("Alias registered: {}\r\nExpands to: {}", alias, &string);
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let mut console = sys.get_mut(eworld);
		console.register_alias(alias.clone(), string.clone());
	})
}

/// Echoes every launch argument given to the client.
pub(crate) fn ccmd_args(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Prints out all of the program's launch arguments.");
	}

	let mut args = env::args();

	let argv0 = match args.next() {
		Some(a) => a,
		None => {
			error!("this runtime did not receive `argv[0]`");
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
pub(crate) fn ccmd_clear(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Clears the console's message history.");
	}

	req_callback(|eworld| {
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let mut console = sys.get_mut(eworld);
		console.clear_message_history(true, true, true);
	})
}

pub(crate) fn ccmd_exit(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Instantly closes the client.");
	}

	req_callback(|eworld| {
		let mut sys: SystemState<EventWriter<AppExit>> = SystemState::new(eworld);
		let mut exit = sys.get_mut(eworld);
		exit.send(AppExit);
	})
}

/// Clears the console's history of submitted input strings.
pub(crate) fn ccmd_hclear(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help("Clear's the console's history of submitted input strings.");
	}

	req_callback(|eworld| {
		info!("Clearing history of submitted input strings.");
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let mut console = sys.get_mut(eworld);
		console.clear_input_history();
	})
}

/// Prints a list of all available console commands if given no arguments.
/// If the first argument is a command's name, it's equivalent to submitting
/// `command --help`.
pub(crate) fn ccmd_help(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"If used without arguments, prints a list of all available commands.\r\n\
			Giving the name of a command as a first argument is the same as giving \
			`command --help`.",
		);
	}

	if args.name_only() {
		return req_callback(|eworld| {
			let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
			let mut console = sys.get_mut(eworld);

			let cap = console.all_commands().map(|cmd| cmd.0.len()).sum();
			let mut string = String::with_capacity(cap);

			string.push_str("All available commands:");

			for command in console.all_commands() {
				string.push('\r');
				string.push('\n');
				string.push_str(command.0);
			}

			console.write(string, MessageKind::Help);
		});
	}

	let key = args[1].to_string();

	req_callback(move |eworld| {
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let console = sys.get_mut(eworld);

		match console.find_command(&key) {
			Some(cmd) => {
				(cmd.func)(terminal::CommandArgs::new(vec![&key, "--help"]));
			}
			None => {
				info!("No command found by name: {}", key);
			}
		}
	})
}

/// Prints the full version information of the engine and client.
pub(crate) fn ccmd_version(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(
			"Prints the full version information of the engine and client.",
		);
	}

	let c_vers = env!("CARGO_PKG_VERSION");
	let [e_vers, commit, comp_datetime] = viletech::version_info();

	let msg = formatdoc! {"
VileTech Client {c_vers}
{e_vers}
{commit}
{comp_datetime}"};

	info!("{msg}");

	Request::None
}

/// Prints information about the WGPU render device's features and limits.
pub(crate) fn ccmd_wgpudiag(args: CommandArgs) -> Request {
	if args.help_requested() {
		return req_console_write_help(formatdoc! {
			"Print information about the WGPU render device's features and limits.

			Usage: {} [options]

			Options:
				-d, --default
					Also print the set of limits guaranteed to work on all modern backends.
				-l, --downlevel
					Also print the set of limits guaranteed to work on \"downlevel\"
					backends such as OpenGL and D3D11.
				-w, --webgl2
					Also print the set of limits low enough to support running
					in a brower using WebGL2.",
			args.command_name(),
		});
	}

	let default = args.has_any_option(&["-d", "--default"]);
	let downlevel = args.has_any_option(&["-l", "--downlevel"]);
	let webgl2 = args.has_any_option(&["-w", "--webgl2"]);

	fn print_limits(limits: WgpuLimits, header: &'static str) {
		let msg = formatdoc! {"
			{header}:
				Max. 2D texture width and height: {tex2d_dim}
				Max. push constant size: {pushconst}
				Max. samplers per shader stage: {samplers}
				Max. sampled textures per shader stage: {sampled_tex}
				Max. texture array layers: {tex_arr_layers}
				Max. vertex attributes: {vattrs}",
			tex2d_dim = limits.max_texture_dimension_2d,
			pushconst = limits.max_push_constant_size,
			tex_arr_layers = limits.max_texture_array_layers,
			samplers = limits.max_samplers_per_shader_stage,
			sampled_tex = limits.max_sampled_textures_per_shader_stage,
			vattrs = limits.max_vertex_attributes,
		};

		info!("{msg}");
	}

	req_callback(move |eworld| {
		let mut sys: SystemState<Res<RenderDevice>> = SystemState::new(eworld);
		let rdev = sys.get_mut(eworld);

		let feats = rdev.features();

		let msg = formatdoc! {"
			Current WGPU render device features:
				Multiview
				PolygonMode::Line: {f_linemode}
				PolygonMode::Point: {f_pointmode}
				Push constants: {f_pushconsts}",
			f_linemode = feats.contains(WgpuFeatures::POLYGON_MODE_LINE),
			f_pointmode = feats.contains(WgpuFeatures::POLYGON_MODE_POINT),
			f_pushconsts = feats.contains(WgpuFeatures::PUSH_CONSTANTS),
		};

		info!("{msg}");

		print_limits(rdev.limits(), "Current WGPU render device limits");

		if default {
			print_limits(WgpuLimits::default(), "Default WGPU render device limits");
		}

		if downlevel {
			print_limits(
				WgpuLimits::downlevel_defaults(),
				"Downlevel WGPU render device limits",
			);
		}

		if webgl2 {
			print_limits(
				WgpuLimits::downlevel_webgl2_defaults(),
				"WebGL2 render device limits",
			);
		}
	})
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
#[allow(unused)]
fn req_console_write_invalidopt(opt: &str) -> Request {
	let msg = format!("Unknown option: `{opt}`");

	Request::Callback(Box::new(move |eworld| {
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let mut console = sys.get_mut(eworld);
		console.write(msg.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_console_write_help(message: impl Into<String>) -> Request {
	let message = message.into();

	Request::Callback(Box::new(move |eworld| {
		let mut sys: SystemState<ResMut<Console>> = SystemState::new(eworld);
		let mut console = sys.get_mut(eworld);
		console.write(message.clone(), MessageKind::Help);
	}))
}

#[must_use]
fn req_callback<F: 'static + Fn(&mut World) + Send + Sync>(callback: F) -> Request {
	Request::Callback(Box::new(callback))
}
