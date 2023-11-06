//! A structure for caching incoming mouse/keyboard/etc. input state from winit,
//! for use by the engine's other subsystems.

use std::sync::Arc;

use bevy::{
	input::{
		keyboard::KeyboardInput,
		touchpad::{TouchpadMagnify, TouchpadRotate},
		ButtonState, InputSystem,
	},
	prelude::*,
};
use bevy_egui::systems::InputEvents;
use parking_lot::{Mutex, RwLock};

/// Stateful storage of user input.
#[derive(Debug, Default, Resource)]
pub struct InputCore {
	pub keys_virt: Input<KeyCode>,
	pub keys_phys: Input<ScanCode>,
	pub mouse_buttons: Input<MouseButton>,
	pub gamepad_buttons: Input<GamepadButton>,
	/// In logical pixels.
	pub cursor_pos: Vec2,
	/// In logical pixels.
	pub cursor_pos_prev: Vec2,
}

impl InputCore {
	pub fn update(&mut self, mut events: InputEvents) {
		self.keys_virt.clear();
		self.keys_phys.clear();
		self.mouse_buttons.clear();
		self.gamepad_buttons.clear();

		for event in events.ev_keyboard_input.read() {
			let KeyboardInput {
				scan_code, state, ..
			} = event;

			if let Some(key_code) = event.key_code {
				match state {
					ButtonState::Pressed => self.keys_virt.press(key_code),
					ButtonState::Released => self.keys_virt.release(key_code),
				}
			}

			match state {
				ButtonState::Pressed => self.keys_phys.press(ScanCode(*scan_code)),
				ButtonState::Released => self.keys_phys.release(ScanCode(*scan_code)),
			}
		}

		for event in events.ev_mouse_button_input.read() {
			match event.state {
				ButtonState::Pressed => self.mouse_buttons.press(event.button),
				ButtonState::Released => self.mouse_buttons.release(event.button),
			}
		}

		self.cursor_pos_prev = self.cursor_pos;

		for event in events.ev_cursor.read() {
			self.cursor_pos = event.position;
		}
	}
}

#[derive(Debug)]
pub struct Binding {
	pub id: String,
	pub trigger: BindingTrigger,
	pub trigger_alt: BindingTrigger,
	pub shift: bool,
	pub alt: bool,
	pub ctrl: bool,
	// TODO: What a triggered binding actually does will depend on Lith.
}

#[derive(Debug)]
pub enum BindingTrigger {
	ScanCode(u32),
	MouseButton(MouseButton),
	// TODO: Gamepad support.
}

/// Copies [`bevy::input::InputPlugin`] but adds no input resources and no
/// systems. Instead, [add systems](bevy::prelude::App::add_system) to the
/// [`InputSystem`] set which feeds [`InputEvents`] to an [`InputCore`].
#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		use bevy::input::{gamepad::*, keyboard::*, mouse::*, touch::*};

		app.configure_sets(PreUpdate, InputSystem)
			// keyboard
			.add_event::<KeyboardInput>()
			.init_resource::<Input<KeyCode>>()
			.init_resource::<Input<ScanCode>>()
			// mouse
			.add_event::<MouseButtonInput>()
			.add_event::<MouseMotion>()
			.add_event::<MouseWheel>()
			.init_resource::<Input<MouseButton>>()
			// gamepad
			.add_event::<GamepadConnectionEvent>()
			.add_event::<GamepadButtonChangedEvent>()
			.add_event::<GamepadAxisChangedEvent>()
			.add_event::<GamepadEvent>()
			.init_resource::<GamepadSettings>()
			.init_resource::<Gamepads>()
			.init_resource::<Input<GamepadButton>>()
			.init_resource::<Axis<GamepadAxis>>()
			.init_resource::<Axis<GamepadButton>>()
			// touch
			.add_event::<TouchInput>()
			.add_event::<TouchpadMagnify>()
			.add_event::<TouchpadRotate>()
			.init_resource::<Touches>();

		// Register common types
		app.register_type::<ButtonState>();

		// Register keyboard types
		app.register_type::<KeyboardInput>()
			.register_type::<KeyCode>()
			.register_type::<ScanCode>();

		// Register mouse types
		app.register_type::<MouseButtonInput>()
			.register_type::<MouseButton>()
			.register_type::<MouseMotion>()
			.register_type::<MouseScrollUnit>()
			.register_type::<MouseWheel>();

		// Register touchpad types
		app.register_type::<TouchpadMagnify>()
			.register_type::<TouchpadRotate>();

		// Register touch types
		app.register_type::<TouchInput>()
			.register_type::<ForceTouch>()
			.register_type::<TouchPhase>();

		// Register gamepad types
		app.register_type::<Gamepad>()
			.register_type::<GamepadConnection>()
			.register_type::<GamepadButtonType>()
			.register_type::<GamepadButton>()
			.register_type::<GamepadAxisType>()
			.register_type::<GamepadAxis>()
			.register_type::<GamepadSettings>()
			.register_type::<ButtonSettings>()
			.register_type::<AxisSettings>()
			.register_type::<ButtonAxisSettings>();
	}
}

/// A type alias for convenience and to reduce line noise.
pub type InputCoreAM = Arc<Mutex<InputCore>>;
/// A type alias for convenience and to reduce line noise.
pub type InputCoreAL = Arc<RwLock<InputCore>>;
