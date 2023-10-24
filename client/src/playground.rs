//! The client's Lithica playground.

use std::time::{Duration, Instant};

use bevy::prelude::Resource;
use bevy_egui::egui;
use viletech::lith::{
	self, ariadne,
	compile::{self, LibMeta},
	filetree,
	issue::FileSpan,
	Compiler,
};

#[derive(Resource, Debug)]
pub(crate) struct Playground {
	code: String,
	reports: Vec<String>,
	dev_mode: bool,
	prev_compile_time: Option<Duration>,
}

impl Default for Playground {
	fn default() -> Self {
		Self {
			code: String::new(),
			reports: vec![],
			dev_mode: true,
			prev_compile_time: None,
		}
	}
}

impl Playground {
	pub(crate) fn ui(&mut self, _: &egui::Context, ui: &mut egui::Ui) {
		egui::menu::bar(ui, |ui| {
			if ui.button("\u{25B6} Run").clicked() {
				let start_time = Instant::now();
				self.compile();
				self.prev_compile_time = Some(start_time.elapsed());
				// TODO: check for a `function main` with no arguments or return values.
				// Try executing it.
			}

			ui.separator();

			ui.checkbox(&mut self.dev_mode, "Developer Mode")
				.on_hover_text("If enabled, optimizations will be disabled and developer mode-only code will remain.");
		});

		ui.horizontal_centered(|ui| {
			let avail_w = ui.available_width();

			egui::ScrollArea::vertical()
				.id_source("viletech_playground_code")
				.auto_shrink([false, false])
				.max_width(avail_w / 2.0)
				.show(ui, |ui| {
					ui.add(
						egui::TextEdit::multiline(&mut self.code)
							.font(egui::TextStyle::Monospace) // for cursor height
							.code_editor()
							.desired_rows(20)
							.lock_focus(true)
							.desired_width(f32::INFINITY),
					);
				});

			ui.separator();

			egui::ScrollArea::vertical()
				.id_source("viletech_playground_out")
				.auto_shrink([false, false])
				.max_width(avail_w / 2.0)
				.show(ui, |ui| {
					ui.vertical(|ui| {
						ui.heading("Compiler Output");

						if let Some(p) = self.prev_compile_time {
							ui.label(format!("Compiled in {}ms.", p.as_millis()));

							if self.reports.len() == 1 {
								ui.label(format!("{} compiler error/warning.", self.reports.len()));
							} else {
								ui.label(format!(
									"{} compiler errors/warnings.",
									self.reports.len()
								));
							}
						}

						ui.separator();

						egui::Grid::new("viletech_playground_out_grid")
							.striped(true)
							.show(ui, |ui| {
								for report in &self.reports {
									ui.label(report);
									ui.end_row();
								}
							});
					});
				});
		});
	}

	fn compile(&mut self) {
		self.reports.clear();

		let mut compiler = Compiler::new(compile::Config {
			opt: if self.dev_mode {
				lith::OptLevel::None
			} else {
				lith::OptLevel::SpeedAndSize
			},
			hotswap: false,
		});

		let result = compiler.register_lib(
			LibMeta {
				name: "playground".to_string(),
				version: lith::Version::V0_0_0,
				native: false,
			},
			|ftree| {
				let folder_ix = ftree.add_folder(ftree.root(), "playground");
				let file_ix = ftree.add_file(folder_ix, "playground.lith", &self.code);

				let filetree::Node::File { ptree, .. } = ftree.get(file_ix).unwrap() else {
					unreachable!()
				};

				if ptree.any_errors() {
					let mut src = Source(ariadne::Source::from(self.code.clone()));

					for err in ptree.errors() {
						let mut msg = format!("parser found: {}\r\n", err.found().token());
						msg.push_str("expected one of the following:");

						for expected in err.expected() {
							msg.push_str("\r\n");
							msg.push_str("- ");
							msg.push_str(expected);
						}

						let builder = ariadne::Report::build(
							ariadne::ReportKind::Error,
							"/playground/playground.lith".to_string(),
							err.found().span().start,
						)
						.with_config(ariadne::Config::default().with_color(false))
						.with_message(msg);
						// TODO: egui-side coloration.

						let report: ariadne::Report<'_, FileSpan> = builder.finish();
						let mut buf = vec![];
						report.write(&mut src, &mut buf).unwrap();
						self.reports.push(String::from_utf8(buf).unwrap());
					}

					return Err(vec![lith::Error::Parse]);
				}

				Ok(folder_ix)
			},
		);

		if result.is_err() {
			return;
		}

		compiler.finish_registration();

		compile::declare_symbols(&mut compiler);

		if compiler.failed() {
			self.generate_reports(compiler);
			return;
		}

		compile::resolve_imports(&mut compiler);

		if compiler.failed() {
			self.generate_reports(compiler);
			return;
		}

		compile::semantic_check(&mut compiler);

		if compiler.failed() {
			self.generate_reports(compiler);
			return;
		}

		let _ = compile::finalize(compiler, true, true);
	}

	fn generate_reports(&mut self, mut compiler: Compiler) {
		let mut src = Source(ariadne::Source::from(self.code.clone()));

		for iss in compiler.drain_issues() {
			let report = iss.report();
			let mut buf = vec![];
			report.write(&mut src, &mut buf).unwrap();
			self.reports.push(String::from_utf8(buf).unwrap());
		}
	}
}

struct Source(ariadne::Source);

impl ariadne::Cache<str> for Source {
	fn fetch(&mut self, _: &str) -> Result<&ariadne::Source, Box<dyn std::fmt::Debug + '_>> {
		Ok(&self.0)
	}

	fn display<'a>(&self, id: &'a str) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new(id))
	}
}
