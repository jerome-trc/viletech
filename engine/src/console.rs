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

use crossbeam::channel::{Receiver, Sender};
use egui::{
	text::{CCursor, LayoutJob},
	text_edit::{CCursorRange, TextEditState},
	Color32, TextFormat,
};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::{cell::Cell, collections::VecDeque, io, path::PathBuf, thread, time::Duration};
use winit::event::{KeyboardInput, VirtualKeyCode};

pub struct Console {
	open: Cell<bool>,
	/// Takes messages written by logging to fill `log`.
	receiver: Receiver<String>,
	/// Output from the `log` crate, displayed to the user.
	/// Also includes every line of input submitted.
	log: Vec<String>,
	/// Each element is a line of input submitted. Allows the user to scroll
	/// back through previous inputs with the up and down arrow keys.
	history: Vec<String>,
	aliases: Vec<ConsoleAlias>,
	commands: Vec<ConsoleCommand>,
	/// The currently-buffered input waiting to be submitted.
	input: String,

	history_pos: usize,
	defocus_textedit: bool,
	scroll_to_bottom: bool,
	cursor_to_end: bool,

	pub requests: VecDeque<ConsoleRequest>,
}

impl Console {
	pub fn new(msg_receiver: Receiver<String>) -> Self {
		Console {
			#[cfg(debug_assertions)]
			open: Cell::new(true),
			#[cfg(not(debug_assertions))]
			open: Cell::new(false),
			receiver: msg_receiver,
			log: Vec::<String>::default(),
			history: Vec::<String>::default(),
			aliases: Vec::<ConsoleAlias>::default(),
			commands: Vec::<ConsoleCommand>::default(),
			input: String::with_capacity(512),
			history_pos: 0,
			defocus_textedit: false,
			scroll_to_bottom: false,
			cursor_to_end: false,
			requests: VecDeque::<ConsoleRequest>::default(),
		}
	}

	fn find_command(&self, key: &str) -> Option<&ConsoleCommand> {
		for cmd in &self.commands {
			if !key.eq_ignore_ascii_case(cmd.id) {
				continue;
			}

			return Some(cmd);
		}

		None
	}

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

		lazy_static! {
			static ref RGX_ARGSPLIT: Regex = Regex::new(r#"'([^']+)'|"([^"]+)"|([^'" ]+) *"#)
				.expect("Failed to evaluate `Console::try_submit::RGX_ARGSPLIT`.");
		};

		let mut tokens = self.input.splitn(2, ' ');
		let key = if let Some(k) = tokens.next() {
			k
		} else {
			return;
		};

		if key.eq_ignore_ascii_case("clear") {
			self.input.clear();
			self.log.clear();
			return;
		} else if key.eq_ignore_ascii_case("clearhist") {
			info!("History of submitted commands cleared.");
			self.input.clear();
			self.history.clear();
			self.history_pos = 0;
			return;
		}

		let args = if let Some(a) = tokens.next() { a } else { "" };
		let args_iter = RGX_ARGSPLIT.captures_iter(args);
		let mut args = Vec::<&str>::default();

		for arg in args_iter {
			let arg_match = match arg.get(1).or_else(|| arg.get(2)).or_else(|| arg.get(3)) {
				Some(a) => a,
				None => {
					continue;
				}
			};

			args.push(arg_match.as_str());
		}

		info!("$ {}", self.input);

		let help = key.eq_ignore_ascii_case("help") || key == "?";
		let mut cmd_found = false;

		if help {
			if !args.is_empty() {
				match self.find_command(args[0]) {
					Some(cmd) => {
						cmd_found = true;
						(cmd.help)(cmd, args);
					}
					None => {}
				};
			} else {
				self.log.push("All available commands:".to_string());

				for cmd in &self.commands {
					self.log.push(cmd.id.to_string());
				}

				cmd_found = true;
			}
		} else {
			match self.find_command(key) {
				Some(cmd) => {
					cmd_found = true;

					match (cmd.func)(cmd, args) {
						ConsoleRequest::None => {}
						req => {
							self.requests.push_front(req);
						}
					};
				}
				None => {}
			}
		}

		for alias in &self.aliases {
			if !key.eq_ignore_ascii_case(&alias.0) {
				continue;
			}

			// TODO
		}

		if !cmd_found {
			info!("Unknown command: {}", key);
		}

		self.scroll_to_bottom = true;
		self.input.clear();
	}

	fn draw_log_line(&self, ui: &mut egui::Ui, line: &str) {
		lazy_static! {
			static ref RGX_LOGOUTPUT: Regex =
				Regex::new(r"^\[[A-Z]+\] ").expect("Failed to evaluate `RGX_LOGOUTPUT`.");
		};

		if !RGX_LOGOUTPUT.is_match(line) {
			ui.label(line);
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

		job.append("[", 0.0, TextFormat::default());
		job.append(
			&loglvl[1..],
			0.0,
			TextFormat {
				color,
				..Default::default()
			},
		);
		job.append("] ", 0.0, TextFormat::default());
		job.append(s.next().unwrap_or_default(), 0.0, TextFormat::default());

		let galley = ui.fonts().layout_job(job);
		ui.label(galley);
	}

	fn draw_impl(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
		let scroll_area = egui::ScrollArea::vertical()
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

	pub fn register_command(&mut self, cmd: ConsoleCommand) {
		self.commands.push(cmd);
	}

	pub fn draw(&mut self, ctx: &egui::Context) {
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
				self.draw_impl(ui, ctx);
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
}

pub enum ConsoleRequest {
	None,
	// Client-fulfilled requests
	LuaMem,
	File(PathBuf),
	Sound(String),
	Uptime,
}

pub struct ConsoleCommand {
	id: &'static str,
	/// The vector of arguments never contains the name of the command itself,
	/// whether aliased or not.
	func: fn(&Self, Vec<&str>) -> ConsoleRequest,
	help: fn(&Self, Vec<&str>),
	/// If false, this command absolutely cannot be executed via script.
	script_legal: bool,
}

impl ConsoleCommand {
	pub fn new(
		id: &'static str,
		func: fn(&Self, Vec<&str>) -> ConsoleRequest,
		help: fn(&Self, Vec<&str>),
		script_legal: bool,
	) -> Self {
		ConsoleCommand {
			id,
			func,
			help,
			script_legal,
		}
	}

	pub fn get_id(&self) -> &'static str {
		self.id
	}

	/// Allows contexts outside this module to register `func` callbacks
	///  which print out help without increasing visibility any further.
	pub fn call_help(&self, args: Option<Vec<&str>>) {
		match args {
			Some(a) => ((self.help)(self, a)),
			None => ((self.help)(self, Vec::<&str>::default())),
		}
	}
}

type ConsoleAlias = (String, &'static str);

pub struct ConsoleWriter {
	buffer: Vec<u8>,
	sender: Sender<String>,
}

impl ConsoleWriter {
	pub fn new(sender: Sender<String>) -> Self {
		ConsoleWriter {
			buffer: Vec::<u8>::with_capacity(512),
			sender,
		}
	}
}

impl io::Write for ConsoleWriter {
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
