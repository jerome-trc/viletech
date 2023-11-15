//! Functions run when entering, updating, and leaving [`AppState::Frontend`].

use std::path::PathBuf;

use bevy::{app::AppExit, prelude::*};
use viletech::{
	frontend::{FrontendMenu, LoadOrderEntryKind, Outcome},
	user::UserCore,
	vfs::VPath,
};

use crate::{common::ClientCommon, load::GameLoad, AppState};

// Bevy systems ////////////////////////////////////////////////////////////////

pub(crate) fn update(
	mut cmds: Commands,
	mut core: ClientCommon,
	mut next_state: ResMut<NextState<AppState>>,
	mut frontend: ResMut<FrontendMenu>,
	user: ResMut<UserCore>,
	mut exit: EventWriter<AppExit>,
) {
	let action = frontend.ui(core.egui.ctx_mut());

	match action {
		Outcome::None => {}
		Outcome::StartGame => {
			if validate_load_order(&frontend) {
				let to_mount = frontend.to_mount();
				let to_mount = to_mount.into_iter().map(|p| p.to_path_buf()).collect();

				cmds.insert_resource(
					start_load(&core, to_mount, frontend.dev_mode()).unwrap_or_else(|_| {
						unimplemented!("handling load order errors is currently unimplemented")
					}),
				);

				next_state.set(AppState::Load);
			}
		}
		Outcome::StartEditor => {
			if validate_load_order(&frontend) {
				let to_mount = frontend.to_mount();

				for path in to_mount {
					let fname = path.file_name().unwrap(/* TODO */);
					let s = fname.to_string_lossy();
					let mpoint = VPath::new(s.as_ref());
					core.vfs.mount(path, mpoint).unwrap(/* TODO */);
					info!("Mounted `{}` to `/{mpoint}`.", path.display());
				}

				core.vfs.ingest_all();

				next_state.set(AppState::Editor);
			}
		}
		Outcome::Exit => {
			exit.send(AppExit);
			on_exit(cmds, frontend, user);
		}
	}
}

pub(crate) fn on_enter(mut cmds: Commands, user: ResMut<UserCore>) {
	let globalcfg = user.globalcfg();

	cmds.insert_resource(FrontendMenu::new(
		Some((
			globalcfg.load_order_presets.clone(),
			globalcfg.cur_load_order_preset,
		)),
		globalcfg.dev_mode,
	));
}

pub(crate) fn on_exit(
	mut cmds: Commands,
	mut frontend: ResMut<FrontendMenu>,
	mut user: ResMut<UserCore>,
) {
	cmds.remove_resource::<FrontendMenu>();

	let globalcfg = user.globalcfg_mut();
	globalcfg.dev_mode = frontend.dev_mode();

	let (loadord_presets, cur_preset) = frontend.consume();
	globalcfg.load_order_presets = loadord_presets;
	globalcfg.cur_load_order_preset = cur_preset;

	if let Err(err) = user.write_global_cfg() {
		error!(
			"Failed to write to global config file: {p}\r\n\tDetails: {err}",
			p = user.globalcfg_path().display()
		);
	}
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn validate_load_order(frontend: &FrontendMenu) -> bool {
	let mut all_valid = true;

	for loi in frontend.load_order().iter() {
		let LoadOrderEntryKind::Item { path, enabled } = &loi.kind else {
			continue;
		};

		if !*enabled {
			continue;
		}

		if !path.exists() {
			all_valid = false;
			info!(
				"`{}` does not exist; it may have been moved or deleted.",
				path.display()
			);
		}
	}

	all_valid
}

fn start_load(_: &ClientCommon, _: Vec<PathBuf>, _: bool) -> Result<GameLoad, String> {
	unimplemented!("new game loading scheme under development")

	/*

	let start_time = Instant::now();
	let catalog = core.catalog();
	let tracker_m = Arc::new(SendTracker::default());
	let tracker_p = Arc::new(SendTracker::default());

	to_mount.dedup();

	let mut load_order = Vec::with_capacity(to_mount.len());

	for real_path in to_mount {
		debug_assert!(real_path.exists());

		if real_path.is_symlink() {
			return Err(format!(
				"Could not mount file: {}\r\n\t\
				Details: mounting symbolic links is forbidden.",
				real_path.display()
			));
		}

		let fstem = if let Some(stem) = real_path.file_stem() {
			stem
		} else {
			return Err(format!(
				"Could not mount file: {}\r\n\t\
				Details: file has no name.",
				real_path.display()
			));
		};

		let mount_point = if let Some(s) = fstem.to_str() {
			s
		} else {
			return Err(format!(
				"Could not mount file: {}\r\n\t\
				Details: file has invalid characters in its name.",
				real_path.display()
			));
		};

		let mount_point = VPathBuf::from(mount_point);
		load_order.push((real_path, mount_point));
	}

	let load_order_sent = load_order.clone();
	let tracker_m_sent = tracker_m.clone();
	let tracker_p_sent = tracker_p.clone();

	let thread = std::thread::spawn(move || {
		let request = LoadRequest {
			mount: MountRequest {
				load_order: load_order_sent,
				tracker: Some(tracker_m_sent),
				basedata: false,
			},
			tracker: Some(tracker_p_sent),
			dev_mode,
		};

		todo!()
	});

	Ok(GameLoad {
		thread: Some(thread),
		tracker_m,
		tracker_p,
		start_time,
		load_order,
	})

	*/
}
