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

use std::{cell::Cell, collections::VecDeque, io, thread, time::Duration};

use crossbeam::channel::{Receiver, Sender};
use egui::{
	text::{CCursor, LayoutJob},
	text_edit::{CCursorRange, TextEditState},
	Color32, ScrollArea, TextFormat, TextStyle,
};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use winit::event::{KeyboardInput, VirtualKeyCode};

use crate::terminal::{self, Alias, Terminal};

pub struct Console<C: terminal::Command> {
	open: Cell<bool>,
	/// Takes messages written by logging to fill `log`.
	receiver: Receiver<String>,
	/// Output from the `log` crate, displayed to the user.
	/// Also includes every line of input submitted.
	log: Vec<String>,
	/// Each element is a line of input submitted. Allows the user to scroll
	/// back through previous inputs with the up and down arrow keys.
	history: Vec<String>,
	terminal: Terminal<C>,
	/// The currently-buffered input waiting to be submitted.
	input: String,

	history_pos: usize,
	defocus_textedit: bool,
	scroll_to_bottom: bool,
	cursor_to_end: bool,

	pub requests: VecDeque<C::Output>,
}

// Public interface.
impl<C: terminal::Command> Console<C> {
	#[must_use]
	pub fn new(msg_receiver: Receiver<String>) -> Self {
		Console {
			#[cfg(debug_assertions)]
			open: Cell::new(true),
			#[cfg(not(debug_assertions))]
			open: Cell::new(false),
			receiver: msg_receiver,
			log: Vec::<String>::default(),
			history: Vec::<String>::default(),
			terminal: Terminal::<C>::new(|key| {
				info!("Unknown command: {}", key);
			}),
			input: String::with_capacity(512),
			history_pos: 0,
			defocus_textedit: false,
			scroll_to_bottom: false,
			cursor_to_end: false,
			requests: VecDeque::<C::Output>::default(),
		}
	}

	pub fn ui(&mut self, ctx: &egui::Context) {
		let mut recvtries: u8 = 0;

		while !self.receiver.is_empty() && recvtries < 100 {
			let s = match self.receiver.recv() {
				Ok(s) => s,
				Err(err) => {
					error!(
						"Console message channel was disconnected unexpectedly: {}",
						err
					);
					recvtries += 1;
					continue;
				}
			};

			if !s.is_empty() {
				self.log.push(s);
			}
		}

		if !self.open.get() {
			return;
		}

		let mut o = self.open.get();

		egui::Window::new("Console")
			.open(&mut o)
			.resizable(true)
			.show(ctx, |ui| {
				self.ui_impl(ui, ctx);
			});

		self.open.set(o);
	}

	pub fn on_key_event(&mut self, input: &KeyboardInput) {
		if input.state != winit::event::ElementState::Pressed {
			return;
		}

		let vkc = match input.virtual_keycode {
			Some(kc) => kc,
			None => {
				return;
			}
		};

		if !self.open.get() {
			if vkc == VirtualKeyCode::Grave {
				self.open.set(true);
			} else {
				return;
			}
		} else if vkc == VirtualKeyCode::Grave {
			self.open.set(false);
			return;
		}

		match input.virtual_keycode.unwrap() {
			VirtualKeyCode::Escape => {
				self.defocus_textedit = true;
			}
			VirtualKeyCode::Return => self.try_submit(),
			VirtualKeyCode::Up => {
				if self.history_pos < 1 {
					return;
				}

				self.cursor_to_end = true;
				self.history_pos -= 1;
				self.input.clear();
				self.input.push_str(&self.history[self.history_pos]);
			}
			VirtualKeyCode::Down => {
				if self.history_pos >= self.history.len() {
					return;
				}

				self.cursor_to_end = true;
				self.history_pos += 1;
				self.input.clear();

				if self.history_pos < self.history.len() {
					self.input.push_str(&self.history[self.history_pos]);
				}
			}
			_ => {}
		}
	}

	pub fn register_command(&mut self, id: &'static str, cmd: C, enabled: bool) {
		self.terminal.register_command(id, cmd, enabled);
	}

	pub fn register_alias(&mut self, alias: String, string: String) {
		self.terminal.register_alias(alias, string);
	}

	pub fn enable_commands(&mut self, predicate: fn(&C) -> bool) {
		self.terminal.enable_commands(predicate);
	}

	pub fn disable_commands(&mut self, predicate: fn(&C) -> bool) {
		self.terminal.disable_commands(predicate);
	}

	pub fn enable_all_commands(&mut self) {
		self.terminal.enable_all_commands();
	}

	pub fn disable_all_commands(&mut self) {
		self.terminal.disable_all_commands();
	}

	pub fn all_commands(&self) -> impl Iterator<Item = (&'static str, &C)> {
		self.terminal.all_commands()
	}

	pub fn all_aliases(&self) -> impl Iterator<Item = &Alias> {
		self.terminal.all_aliases()
	}

	#[must_use]
	pub fn find_command(&self, key: &str) -> Option<&C> {
		self.terminal.find_command(key)
	}

	#[must_use]
	pub fn find_alias(&self, key: &str) -> Option<&Alias> {
		self.terminal.find_alias(key)
	}

	pub fn clear_log(&mut self) {
		self.log.clear();
	}

	pub fn clear_input_history(&mut self) {
		self.history.clear();
		self.history_pos = 0;
	}
}

// Internal implementation details.
impl<C: terminal::Command> Console<C> {
	fn try_submit(&mut self) {
		if self.input.is_empty() {
			info!("$");
			self.scroll_to_bottom = true;
			return;
		}

		match self.history.last() {
			Some(last_cmd) => {
				if last_cmd != &self.input[..] {
					self.history.push(self.input.clone());
					self.history_pos = self.history.len();
				}
			}
			None => {
				self.history.push(self.input.clone());
				self.history_pos = self.history.len();
			}
		};

		info!("$ {}", &self.input);
		let mut ret = self.terminal.submit(&self.input);

		for output in ret.drain(..) {
			self.requests.push_back(output);
		}

		self.scroll_to_bottom = true;
		self.input.clear();
	}

	fn draw_log_line(&self, ui: &mut egui::Ui, line: &str) {
		lazy_static! {
			static ref RGX_LOGOUTPUT: Regex =
				Regex::new(r"^\[[A-Z]+\] ").expect("Failed to evaluate `RGX_LOGOUTPUT`.");
		};

		let font_id = TextStyle::Monospace.resolve(ui.style());

		if !RGX_LOGOUTPUT.is_match(line) {
			let job = LayoutJob::simple_singleline(line.to_string(), font_id, Color32::GRAY);
			let galley = ui.fonts().layout_job(job);
			ui.label(galley);
			return;
		}

		let mut job = LayoutJob::default();
		let mut s = line.splitn(2, "] ");

		let loglvl = s.next().unwrap_or_default();

		let color = match &loglvl[1..] {
			"INFO" => Color32::GREEN,
			"WARN" => Color32::YELLOW,
			"ERROR" => Color32::RED,
			_ => Color32::LIGHT_BLUE,
		};

		let tfmt = TextFormat::simple(font_id.clone(), Color32::GRAY);

		job.append("[", 0.0, tfmt.clone());
		job.append(&loglvl[1..], 0.0, TextFormat::simple(font_id, color));
		job.append("] ", 0.0, tfmt.clone());
		job.append(s.next().unwrap_or_default(), 0.0, tfmt);

		let galley = ui.fonts().layout_job(job);
		ui.label(galley);
	}

	fn ui_impl(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let scroll_area = ScrollArea::vertical()
			.max_height(200.0)
			.auto_shrink([false; 2]);

		scroll_area.show(ui, |ui| {
			ui.vertical(|ui| {
				for item in &self.log {
					for line in item.lines() {
						self.draw_log_line(ui, line);
					}
				}
			});

			if self.scroll_to_bottom {
				self.scroll_to_bottom = false;
				ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
			}
		});

		ui.separator();

		ui.horizontal(|ui| {
			let input_len = self.input.len();
			let edit_id = egui::Id::new("console_text_edit");
			let resp_edit = ui.add(egui::TextEdit::singleline(&mut self.input).id(edit_id));
			let mut tes = egui::TextEdit::load_state(ctx, edit_id).unwrap_or_default();

			if self.cursor_to_end {
				self.cursor_to_end = false;
				let range = CCursorRange::one(CCursor::new(input_len));
				tes.set_ccursor_range(Some(range));
				TextEditState::store(tes, ctx, edit_id);
			}

			if self.defocus_textedit {
				self.defocus_textedit = false;
				resp_edit.surrender_focus();
			}

			if ui.add(egui::widgets::Button::new("Submit")).clicked() {
				self.try_submit();
			}
		});
	}
}

pub struct Writer {
	buffer: Vec<u8>,
	sender: Sender<String>,
}

impl Writer {
	#[must_use]
	pub fn new(sender: Sender<String>) -> Self {
		Writer {
			buffer: Vec::<u8>::with_capacity(512),
			sender,
		}
	}
}

impl io::Write for Writer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if buf[0] == 10 {
			let drain = self.buffer.drain(..);
			let s = String::from_utf8_lossy(drain.as_slice());

			match self.sender.try_send(s.to_string()) {
				Ok(()) => {}
				Err(err) => {
					error!(
						"Console message channel was disconnected unexpectedly: {}",
						err
					);
				}
			}
		} else {
			self.buffer.extend_from_slice(buf);
		}

		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		let drain = self.buffer.drain(..);
		let s = String::from_utf8_lossy(drain.as_slice());

		match self.sender.try_send(s.to_string()) {
			Ok(()) => {}
			Err(err) => {
				error!(
					"Console message channel was disconnected unexpectedly: {}",
					err
				);
			}
		};

		while self.sender.is_full() {
			thread::sleep(Duration::from_millis(10))
		}

		Ok(())
	}
}
