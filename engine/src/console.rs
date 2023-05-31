//! egui-based graphical shell for diagnosis, development, and debugging.

use std::{collections::VecDeque, io, thread, time::Duration};

use bevy::prelude::*;
use bevy_egui::egui::{
	self,
	text::{CCursor, LayoutJob},
	text_edit::{CCursorRange, TextEditState},
	Color32, ScrollArea, TextFormat, TextStyle,
};
use crossbeam::channel::Receiver;
use util::lazy_regex;

use crate::{
	input::InputCore,
	terminal::{self, Alias, Terminal},
};

pub type Sender = crossbeam::channel::Sender<Message>;

/// egui-based graphical shell for diagnosis, development, and debugging.
#[derive(Debug)]
pub struct Console<C: terminal::Command> {
	/// Takes messages coming from the `log` crate's backend.
	log_receiver: Receiver<Message>,
	messages: Vec<Message>,
	/// Each element is a line of input submitted. Allows the user to scroll
	/// back through previous inputs with the up and down arrow keys.
	input_history: Vec<Box<str>>,
	/// Commands, aliases, command string parser.
	terminal: Terminal<C>,
	/// The currently-buffered input waiting to be submitted.
	input: String,

	input_history_pos: usize,
	defocus_textedit: bool,
	scroll_to_bottom: bool,
	cursor_to_end: bool,

	/// If `false`, messages tagged [`MessageKind::Log`] aren't drawn.
	draw_log: bool,
	/// If `false`, messages tagged [`MessageKind::Toast`] aren't drawn.
	draw_toast: bool,

	/// Console commands can emit a "request" in order to act upon the client.
	/// Between frames, this container gets drained and all requests are fulfilled.
	pub requests: VecDeque<C::Output>,
}

/// All messages that get sent to the console are tagged so they can be filtered.
#[derive(Debug, PartialEq, Eq)]
pub enum MessageKind {
	/// Help messages emitted by console usage. These don't go through logging
	/// so as not to pollute stdout/stderr or the log files.
	Help,
	/// Game messages like "you picked up a thing".
	/// Kept separate to enable filtering in the GUI.
	Toast,
	/// `tracing` messages end up stored as one of these.
	Log,
}

#[derive(Debug)]
pub struct Message {
	string: Box<str>,
	kind: MessageKind,
}

// Public interface.
impl<C: terminal::Command> Console<C> {
	#[must_use]
	pub fn new(log_receiver: Receiver<Message>) -> Self {
		Console {
			log_receiver,
			messages: vec![],
			input_history: vec![],
			terminal: Terminal::new(|key| {
				info!("Unknown command: {}", key);
			}),
			input: String::with_capacity(128),
			input_history_pos: 0,
			defocus_textedit: false,
			scroll_to_bottom: false,
			cursor_to_end: false,
			draw_log: true,
			draw_toast: true,
			requests: VecDeque::default(),
		}
	}

	/// Receive incoming messages from the log backend - and ongoing playsim, if
	/// any - and draw the window and all of its contents.
	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		while let Ok(msg) = self.log_receiver.try_recv() {
			self.scroll_to_bottom = true;
			self.messages.push(msg);
		}

		egui::menu::bar(ui, |ui| {
			ui.toggle_value(&mut self.draw_log, "Show Engine Log");
			ui.toggle_value(&mut self.draw_toast, "Show Game Log");
		});

		ui.separator();

		let scroll_area = ScrollArea::both().id_source("vile_devgui_console_scroll");

		scroll_area.show(ui, |ui| {
			ui.vertical(|ui| {
				for item in &self.messages {
					match item.kind {
						MessageKind::Toast => {
							if !self.draw_toast {
								continue;
							}

							for line in item.string.lines() {
								Self::draw_line(ui, line);
							}
						}
						MessageKind::Log => {
							if !self.draw_log {
								continue;
							}

							for line in item.string.lines() {
								Self::draw_line_log(ui, line);
							}
						}
						MessageKind::Help => {
							for line in item.string.lines() {
								Self::draw_line(ui, line);
							}
						}
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

	/// Appends a custom message.
	pub fn write(&mut self, string: String, kind: MessageKind) {
		self.messages.push(Message {
			string: string.into_boxed_str(),
			kind,
		});

		self.scroll_to_bottom = true;
	}

	pub fn input(&mut self, input: &InputCore) {
		if input.keys_virt.pressed(KeyCode::Up) {
			if self.input_history_pos < 1 {
				return;
			}

			self.cursor_to_end = true;
			self.input_history_pos -= 1;
			self.input.clear();
			self.input
				.push_str(&self.input_history[self.input_history_pos]);
		} else if input.keys_virt.pressed(KeyCode::Down) {
			if self.input_history_pos >= self.input_history.len() {
				return;
			}

			self.cursor_to_end = true;
			self.input_history_pos += 1;
			self.input.clear();

			if self.input_history_pos < self.input_history.len() {
				self.input
					.push_str(&self.input_history[self.input_history_pos]);
			}
		} else if input.keys_virt.just_pressed(KeyCode::Escape) {
			self.defocus_textedit = true;
		} else if input.keys_virt.just_pressed(KeyCode::Return) {
			self.try_submit();
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

	pub fn clear_message_history(&mut self, log: bool, toast: bool, help: bool) {
		debug_assert!(
			log || toast || help,
			"Invalid arguments given to `Console::clear_message_history`."
		);

		self.messages.retain(|msg| {
			if msg.kind == MessageKind::Log && log {
				return false;
			}

			if msg.kind == MessageKind::Toast && toast {
				return false;
			}

			if msg.kind == MessageKind::Help && help {
				return false;
			}

			true
		})
	}

	pub fn clear_input_history(&mut self) {
		self.input_history.clear();
		self.input_history_pos = 0;
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

		match self.input_history.last() {
			Some(last_cmd) => {
				if last_cmd.as_ref() != self.input {
					self.input_history.push(self.input.clone().into_boxed_str());
					self.input_history_pos = self.input_history.len();
				}
			}
			None => {
				self.input_history.push(self.input.clone().into_boxed_str());
				self.input_history_pos = self.input_history.len();
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

	fn draw_line(ui: &mut egui::Ui, line: &str) {
		let font_id = TextStyle::Monospace.resolve(ui.style());
		let job =
			LayoutJob::simple_singleline(line.to_string(), font_id, ui.visuals().text_color());
		let galley = ui.fonts(|f| f.layout_job(job));
		ui.label(galley);
	}

	fn draw_line_log(ui: &mut egui::Ui, line: &str) {
		let smatch =
			if let Some(m) = lazy_regex!(r"(  INFO|  WARN| ERROR| DEBUG| TRACE) ").find(line) {
				m
			} else {
				Self::draw_line(ui, line);
				return;
			};

		const INFO_COLOR: Color32 = Color32::from_rgb(45, 189, 122);
		const DEBUG_COLOR: Color32 = Color32::from_rgb(63, 143, 206);
		const TRACE_COLOR: Color32 = Color32::from_rgb(202, 101, 202);

		let color = match smatch.as_str() {
			"  INFO " => INFO_COLOR,
			"  WARN " => Color32::YELLOW,
			" ERROR " => Color32::LIGHT_RED,
			" DEBUG " => DEBUG_COLOR,
			" TRACE " => TRACE_COLOR,
			other => unreachable!("Unexpected log message qualifier: {other}"),
		};

		let mut job = LayoutJob::default();
		let font_id = TextStyle::Monospace.resolve(ui.style());
		let tfmt = TextFormat::simple(font_id.clone(), ui.visuals().text_color());

		job.append(&line[..smatch.start()], 0.0, tfmt.clone());
		job.append(smatch.as_str(), 0.0, TextFormat::simple(font_id, color));
		job.append(&line[smatch.end()..], 0.0, tfmt);

		let galley = ui.fonts(|fonts| fonts.layout_job(job));
		ui.label(galley);
	}
}

/// Provides a bridge between the logging backend, which needs a channel sender
/// as well as a [`std::io::Write`] implementation, and the console.
#[derive(Debug)]
pub struct Writer {
	buffer: Vec<u8>,
	sender: Sender,
}

impl Writer {
	#[must_use]
	pub fn new(sender: Sender) -> Self {
		Writer {
			buffer: Vec::<u8>::with_capacity(512),
			sender,
		}
	}
}

impl io::Write for Writer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if buf[0] == 10 {
			// Newline
			let drain = self.buffer.drain(..);
			let string = String::from_utf8_lossy(drain.as_slice()).to_string();

			match self.sender.try_send(Message {
				string: string.into_boxed_str(),
				kind: MessageKind::Log,
			}) {
				Ok(()) => {}
				Err(_) => {
					#[cfg(debug_assertions)]
					eprintln!("Console message channel was disconnected unexpectedly.");
					return std::io::stderr().lock().write(buf);
				}
			}
		} else {
			self.buffer.extend_from_slice(buf);
		}

		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		let drain = self.buffer.drain(..);
		let string = String::from_utf8_lossy(drain.as_slice()).to_string();

		match self.sender.try_send(Message {
			string: string.into_boxed_str(),
			kind: MessageKind::Log,
		}) {
			Ok(()) => {}
			Err(_) => {
				#[cfg(debug_assertions)]
				eprintln!("Console message channel was disconnected unexpectedly.");
				return std::io::stderr().lock().flush();
			}
		};

		while self.sender.is_full() {
			thread::sleep(Duration::from_millis(10))
		}

		Ok(())
	}
}
