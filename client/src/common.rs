use bevy::{
	ecs::system::{SystemParam, SystemState},
	input::mouse::MouseMotion,
	prelude::*,
	window::ApplicationLifetime,
	winit::WinitWindows,
};
use bevy_egui::{systems::InputEvents, EguiContexts};
use viletech::{
	audio::AudioCore,
	image,
	vfs::{self, VPath},
	VirtualFs,
};

use crate::{
	ccmd,
	dgui::{self, Console, DevGui},
};

#[derive(SystemParam)]
pub(crate) struct InputParam<'w, 's> {
	pub(crate) keys: Res<'w, Input<KeyCode>>,
	pub(crate) mouse: Res<'w, Input<MouseButton>>,
	pub(crate) events: InputEvents<'w, 's>,
	pub(crate) mouse_motion: EventReader<'w, 's, MouseMotion>,
}

#[derive(SystemParam)]
pub(crate) struct ClientCommon<'w, 's> {
	pub(crate) vfs: ResMut<'w, VirtualFs>,
	pub(crate) _input: InputParam<'w, 's>,
	pub(crate) _audio: ResMut<'w, AudioCore>,
	pub(crate) _console: ResMut<'w, Console>,
	pub(crate) egui: EguiContexts<'w, 's>,
}

#[derive(Event, Debug, Clone)]
pub(crate) struct NewWindow(pub(crate) Entity);

pub(crate) fn update(world: &mut World) {
	loop {
		let mut console = world.get_resource_mut::<Console>().unwrap();

		let Some(req) = console.requests.pop_front() else {
			break;
		};

		match req {
			ccmd::Request::Callback(func) => {
				(func)(world);
			}
			ccmd::Request::None => {}
		}
	}

	let mut sys: SystemState<(
		Commands,
		ClientCommon,
		EventReader<NewWindow>,
		EventReader<ApplicationLifetime>,
		NonSend<WinitWindows>,
	)> = SystemState::new(world);

	let (mut cmds, core, mut new_windows, mut app_events, winits) = sys.get_mut(world);

	for new_window in new_windows.read() {
		dgui::add_to_window(cmds.entity(new_window.0));
		let window_id = winits.entity_to_winit.get(&new_window.0).unwrap();
		let window = winits.windows.get(window_id).unwrap();
		set_window_icon(&core.vfs, window);
	}

	for app_event in app_events.read() {
		match app_event {
			ApplicationLifetime::Suspended => info!("Application suspended..."),
			ApplicationLifetime::Resumed => info!("Application resumed..."),
			ApplicationLifetime::Started => {}
		}
	}
}

pub(crate) fn pre_update(
	windows: Query<(&Window, &DevGui)>,
	mut console: ResMut<Console>,
	input: InputParam,
) {
	if !windows
		.iter()
		.any(|(window, dgui)| window.focused && dgui.open)
	{
		return;
	}

	let up_pressed = input.keys.just_pressed(KeyCode::Up);
	let down_pressed = input.keys.just_pressed(KeyCode::Down);
	let esc_pressed = input.keys.just_pressed(KeyCode::Escape);
	let enter_pressed = input.keys.just_pressed(KeyCode::Return);

	console.key_input(up_pressed, down_pressed, esc_pressed, enter_pressed);
}

pub(crate) fn post_update() {}

pub(crate) fn set_window_icon(vfs: &VirtualFs, window: &winit::window::Window) {
	let path = VPath::new("/viletech/viletech.png");

	let Some(r) = vfs.lookup(path) else {
		error!("Window icon not found.");
		return;
	};

	let vfs::Ref::File(fref) = r else {
		error!("`{path}` is unexpectedly a VFS folder.");
		return;
	};

	let mut guard = fref.lock();

	let bytes = match guard.read() {
		Ok(b) => b,
		Err(err) => {
			error!("Failed to read window icon: {err}");
			return;
		}
	};

	let buf = match image::load_from_memory(&bytes) {
		Ok(b) => b.into_rgba8(),
		Err(err) => {
			error!("Failed to load window icon: {err}");
			return;
		}
	};

	let (w, h) = buf.dimensions();
	let rgba = buf.into_raw();

	let icon = match winit::window::Icon::from_rgba(rgba, w, h) {
		Ok(i) => i,
		Err(err) => {
			error!("Failed to create window icon: {err}");
			return;
		}
	};

	window.set_window_icon(Some(icon));
}
