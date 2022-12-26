//! A structure for caching incoming mouse/keyboard/etc. input state from winit,
//! for use by the engine's other subsystems.

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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use mlua::prelude::*;
use winit::{
	dpi::PhysicalPosition,
	event::{ElementState, KeyboardInput, ModifiersState, MouseButton, ScanCode, VirtualKeyCode},
};

// TODO: Use `std::mem::variant_count` when it's stable
const NUM_VIRTKEYS: usize = winit::event::VirtualKeyCode::Cut as usize;

pub struct InputCore {
	pub keys_phys: [bool; 256],
	pub keys_virt: [bool; NUM_VIRTKEYS],
	/// Left, right, middle, and then 15 auxiliary buttons.
	pub mouse_buttons: [bool; 18],
	pub modifiers: ModifiersState,
	pub cursor_pos: PhysicalPosition<f64>,

	pub user_binds: Vec<UserKeyBind>,
}

impl Default for InputCore {
	fn default() -> Self {
		Self {
			keys_phys: [false; 256],
			keys_virt: [false; NUM_VIRTKEYS],
			mouse_buttons: [false; 18],
			modifiers: ModifiersState::default(),
			cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 },
			user_binds: Vec::<UserKeyBind>::default(),
		}
	}
}

impl InputCore {
	pub fn on_modifiers_changed(&mut self, state: &ModifiersState) {
		self.modifiers = *state;
	}

	pub fn on_key_event(&mut self, event: &KeyboardInput) {
		self.keys_phys[event.scancode as usize] = event.state == ElementState::Pressed;

		if let Some(vkc) = event.virtual_keycode {
			self.keys_virt[vkc as usize] = event.state == ElementState::Pressed;
		}
	}

	pub fn on_cursor_moved(&mut self, position: &PhysicalPosition<f64>) {
		self.cursor_pos.x = position.x;
		self.cursor_pos.y = position.y;
	}

	pub fn on_mouse_input(&mut self, button: &MouseButton, state: &ElementState) {
		match button {
			MouseButton::Left => self.mouse_buttons[0] = *state == ElementState::Pressed,
			MouseButton::Right => self.mouse_buttons[1] = *state == ElementState::Pressed,
			MouseButton::Middle => self.mouse_buttons[2] = *state == ElementState::Pressed,
			MouseButton::Other(index) => {
				if *index < 15 {
					self.mouse_buttons[*index as usize] = *state == ElementState::Pressed;
				}
			}
		}
	}

	pub fn pkey_is_up(&self, scancode: ScanCode) -> bool {
		self.keys_phys[scancode as usize]
	}

	pub fn pkey_is_down(&self, scancode: ScanCode) -> bool {
		self.keys_phys[scancode as usize]
	}

	pub fn vkey_is_up(&self, virtcode: VirtualKeyCode) -> bool {
		self.keys_virt[virtcode as usize]
	}

	pub fn vkey_is_down(&self, virtcode: VirtualKeyCode) -> bool {
		self.keys_virt[virtcode as usize]
	}

	pub fn lmb_down(&self) -> bool {
		self.mouse_buttons[0]
	}

	pub fn rmb_down(&self) -> bool {
		self.mouse_buttons[1]
	}

	pub fn mmb_down(&self) -> bool {
		self.mouse_buttons[2]
	}
}

pub struct KeyBind<A> {
	pub id: String,
	pub name: String,
	pub keycode: VirtualKeyCode,
	pub modifiers: ModifiersState,
	pub on_press: A,
	pub on_release: A,
}

pub type UserKeyBind = KeyBind<LuaRegistryKey>;
pub type IdleKeyBind = KeyBind<()>;
