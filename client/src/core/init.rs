//! [`ClientCore::new`] and its helpers.

use std::{sync::Arc, time::Instant};

use parking_lot::RwLock;
use vile::{
	audio::AudioCore,
	console::Console,
	data::Catalog,
	frontend::FrontendMenu,
	gfx::{camera::Camera, core::GraphicsCore},
	input::InputCore,
	lith,
	rng::RngCore,
	user::UserCore,
	DeveloperGui,
};

use crate::{
	ccmd,
	core::{ClientCore, DevGuiStatus, Scene},
};

impl ClientCore {
	pub fn new(
		start_time: Instant,
		catalog: Catalog,
		gfx: GraphicsCore,
		console: Console<ccmd::Command>,
	) -> Result<Self, Box<dyn std::error::Error>> {
		let user;
		let scene;

		let user_dir_home = vile::user::user_dir_home();
		let user_dir_portable = vile::user::user_dir_portable();
		let user_dir = vile::user::select_user_dir(&user_dir_portable, &user_dir_home);

		match user_dir {
			Some(udir) => {
				scene = Scene::Frontend {
					menu: FrontendMenu::default(),
				};

				user = UserCore::new(udir)?;
			}
			None => {
				scene = Scene::FirstStartup {
					portable: true,
					portable_path: user_dir_portable,
					home_path: user_dir_home,
				};

				user = UserCore::uninit();
			}
		}

		let camera = Camera::new(
			gfx.surface_config.width as f32,
			gfx.surface_config.height as f32,
		);

		let catalog = Arc::new(RwLock::new(catalog));
		let catalog_audio = catalog.clone();

		let mut ret = ClientCore {
			start_time,
			user,
			catalog,
			runtime: Arc::new(RwLock::new(lith::Runtime::default())),
			gfx,
			rng: RngCore::default(),
			audio: AudioCore::new(catalog_audio, None)?,
			input: InputCore::default(),
			console,
			camera,
			devgui: DeveloperGui {
				#[cfg(debug_assertions)]
				open: true,
				#[cfg(not(debug_assertions))]
				open: false,
				left: DevGuiStatus::Vfs,
				right: DevGuiStatus::Console,
			},
			scene,
		};

		ret.register_console_commands();

		Ok(ret)
	}

	fn register_console_commands(&mut self) {
		self.console.register_command(
			"alias",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_alias,
			},
			true,
		);

		self.console.register_command(
			"args",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_args,
			},
			true,
		);

		self.console.register_command(
			"clear",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_clear,
			},
			true,
		);

		self.console.register_command(
			"exit",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_exit,
			},
			true,
		);

		self.console.register_command(
			"hclear",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_hclear,
			},
			true,
		);

		self.console.register_command(
			"help",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_help,
			},
			true,
		);

		self.console.register_command(
			"quit",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_exit,
			},
			true,
		); // Built-in alias for "exit".

		self.console.register_command(
			"uptime",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_uptime,
			},
			true,
		);

		self.console.register_command(
			"wgpudiag",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_wgpudiag,
			},
			true,
		);

		self.console.register_command(
			"version",
			ccmd::Command {
				flags: ccmd::Flags::all(),
				func: ccmd::ccmd_version,
			},
			true,
		);
	}
}
