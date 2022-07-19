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
use egui::{text::LayoutJob, Color32, TextFormat};
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
	defocus_textedit: bool,
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
			defocus_textedit: false,
			requests: VecDeque::<ConsoleRequest>::default(),
		}
	}

	fn find_command(&self, key: &str) -> Option<&ConsoleCommand> {
		for cmd in &self.commands {
			if !key.eq_ignore_ascii_case(cmd.key) {
				continue;
			}

			return Some(cmd);
		}

		None
	}

	fn try_submit(&mut self) {
		if self.input.is_empty() {
			info!("$");
			return;
		}

		self.history.push(self.input.clone());
		let mut tokens = self.input.split(' ');

		let key = if let Some(k) = tokens.next() {
			k
		} else {
			return;
		};

		if key.eq_ignore_ascii_case("clear") {
			self.input.clear();
			self.log.clear();
			return;
		}

		info!("$ {}", self.input);

		let help = key.eq_ignore_ascii_case("help") || key == "?";
		let args: Vec<&str> = tokens.collect();
		let mut cmd_found = false;

		if help {
			if !args.is_empty() {
				match self.find_command(args[1]) {
					Some(cmd) => {
						cmd_found = true;
						(cmd.help)(cmd, args);
					}
					None => {}
				};
			} else {
				self.log.push("All available commands:".to_string());

				for cmd in &self.commands {
					self.log.push(cmd.key.to_string());
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

	fn draw_impl(&mut self, ui: &mut egui::Ui) {
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
		});

		ui.separator();

		ui.horizontal(|ui| {
			if self.defocus_textedit {
				ui.text_edit_singleline(&mut self.input).surrender_focus();
			} else {
				ui.text_edit_singleline(&mut self.input);
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
				self.draw_impl(ui);
			});

		self.open.set(o);
	}

	pub fn on_key_event(&mut self, input: &KeyboardInput) {
		if input.state != winit::event::ElementState::Pressed {
			return;
		}

		if input.virtual_keycode.is_none() {
			return;
		}

		let vkc = input.virtual_keycode.unwrap();

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
			_ => {}
		}
	}
}

pub enum ConsoleRequest {
	None,
	// Client-fulfilled requests
	File(PathBuf), // Requests transmitted to the playsim thread
}

pub struct ConsoleCommand {
	key: &'static str,
	/// The vector of arguments never contains the name of the command itself,
	/// whether aliased or not.
	func: fn(&Self, Vec<&str>) -> ConsoleRequest,
	help: fn(&Self, Vec<&str>),
}

impl ConsoleCommand {
	pub fn new(
		key: &'static str,
		func: fn(&Self, Vec<&str>) -> ConsoleRequest,
		help: fn(&Self, Vec<&str>),
	) -> Self {
		ConsoleCommand { key, func, help }
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
				Ok(_) => {}
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
			Ok(_) => {}
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
