//! [`ClientCore`] functions that respond to [`winit`] events.

use log::error;
use vile::{console, frontend::FrontendAction, sim, DeveloperGui};
use winit::{
	event::{ElementState, KeyboardInput, VirtualKeyCode},
	event_loop::ControlFlow,
};

use crate::{
	ccmd,
	core::{ClientCore, DevGuiStatus, Scene, Transition},
};

impl ClientCore {
	/// - Start a new frame.
	/// - Scene-specific rendering and processing.
	///     - Take messages sent by the ongoing sim thread if there is one.
	/// - Draw developer GUI if it is open.
	/// - Submit frame.
	/// - Change scene if necessary.
	/// - Process requests made by the console.
	/// - Update audio core.
	/// Note that the final order for these operations is still TBD.
	pub fn main_loop(&mut self, control_flow: &mut ControlFlow) {
		let mut next_scene = Transition::None;

		let mut frame = match self.gfx.render_start() {
			Ok(f) => f,
			Err(wgpu::SurfaceError::Lost) => {
				self.gfx.resize(self.gfx.window_size);
				return;
			}
			Err(wgpu::SurfaceError::OutOfMemory) => {
				error!("Insufficient memory to allocate a new WGPU frame.");
				*control_flow = ControlFlow::Exit;
				return;
			}
			Err(err) => {
				error!("${err:?}");
				return;
			}
		};

		// Discard this for now; it will get used later.
		let _ = self.camera.update(frame.delta_time_secs_f32());
		self.gfx.egui_start();

		match &mut self.scene {
			Scene::Game { sim } => {
				if let Some(sim) = sim {
					while let Ok(egress) = sim.receiver.try_recv() {
						match egress {
							sim::OutMessage::Toast(toast) => {
								self.console.write(toast, console::MessageKind::Toast)
							}
						}
					}
				}
			}
			Scene::GameLoad {
				thread, tracker, ..
			} => {
				let m_pct = tracker.mount_progress_percent();
				let p_pct = tracker.pproc_progress_percent();
				let mut cancelled = false;

				egui::Window::new("Loading...")
					.id(egui::Id::new("vile_gameload"))
					.show(&self.gfx.egui.context, |ui| {
						ui.label(&format!("File Mounting: {m_pct}%"));
						ui.label(&format!("Processing: {p_pct}%"));

						if ui.button("Cancel").clicked() {
							cancelled = true;
						}
					});

				if tracker.mount_done() && tracker.pproc_done() && !cancelled {
					debug_assert!(thread.is_finished());
					next_scene = Transition::FinishGameLoad;
				} else if cancelled {
					next_scene = Transition::ReturnToFrontend;
				}
			}
			Scene::Frontend { menu } => {
				let action = menu.ui(&self.gfx.egui.context);

				match action {
					FrontendAction::None => {}
					FrontendAction::Quit => {
						*control_flow = ControlFlow::Exit;
					}
					FrontendAction::Start => {
						let to_mount = menu.to_mount();
						let to_mount = to_mount.into_iter().map(|p| p.to_path_buf()).collect();
						next_scene = Transition::StartGameLoad { to_mount };
					}
				}

				let clear_color = if self.gfx.egui.context.style().visuals.dark_mode {
					wgpu::Color {
						r: 0.0,
						g: 0.0,
						b: 0.0,
						a: 1.0,
					}
				} else {
					wgpu::Color {
						r: 0.9,
						g: 0.9,
						b: 0.9,
						a: 1.0,
					}
				};

				let mut rpass = frame.render_pass(clear_color);

				rpass.set_pipeline(&self.gfx.pipelines[0]);
				rpass.draw(0..3, 0..1);
			}
			Scene::FirstStartup {
				portable,
				portable_path,
				home_path,
			} => {
				match Self::first_startup_screen(
					&self.gfx.egui.context,
					portable,
					portable_path.as_path(),
					home_path,
				) {
					Transition::None => {}
					t @ Transition::FirstTimeFrontend => {
						next_scene = t;
					}
					Transition::Exit => {
						*control_flow = ControlFlow::Exit;
						return;
					}
					_ => unreachable!(),
				}
			}
			Scene::Transition => unreachable!(),
		}

		if self.devgui.open && !matches!(self.scene, Scene::FirstStartup { .. }) {
			let ctx = &self.gfx.egui.context;
			let mut devgui_open = true;
			let screen_rect = ctx.input(|inps| inps.screen_rect);

			DeveloperGui::<DevGuiStatus>::window(ctx)
				.open(&mut devgui_open)
				.show(ctx, |ui| {
					// Prevent window from overflowing off the screen's sides
					ui.set_max_width(screen_rect.width());

					self.devgui.selectors(
						ui,
						&[
							(DevGuiStatus::Console, "Console"),
							(DevGuiStatus::LithRepl, "REPL"),
							(DevGuiStatus::Graphics, "Graphics"),
							(DevGuiStatus::Vfs, "VFS"),
							(DevGuiStatus::Audio, "Audio"),
						],
					);

					self.devgui.panel_left(ctx).show_inside(ui, |ui| {
						match self.devgui.left {
							DevGuiStatus::Console => {
								self.console.ui(ctx, ui);
							}
							DevGuiStatus::LithRepl => {
								// Soon!
							}
							DevGuiStatus::Vfs => {
								self.catalog.read().ui(ctx, ui);
							}
							DevGuiStatus::Graphics => {
								self.gfx.ui(ctx, ui);
							}
							DevGuiStatus::Audio => {
								self.audio.ui(ctx, ui);
							}
						};
					});

					self.devgui.panel_right(ctx).show_inside(ui, |ui| {
						match self.devgui.right {
							DevGuiStatus::Console => {
								self.console.ui(ctx, ui);
							}
							DevGuiStatus::LithRepl => {
								// Soon!
							}
							DevGuiStatus::Vfs => {
								self.catalog.read().ui(ctx, ui);
							}
							DevGuiStatus::Graphics => {
								self.gfx.ui(ctx, ui);
							}
							DevGuiStatus::Audio => {
								self.audio.ui(ctx, ui);
							}
						};
					});
				});

			self.devgui.open = devgui_open;
		}

		self.gfx.render_finish(frame);

		self.scene_change(next_scene);

		while !self.console.requests.is_empty() {
			match self.console.requests.pop_front().unwrap() {
				ccmd::Request::Callback(func) => {
					(func)(self);
				}
				ccmd::Request::Exit => {
					*control_flow = ControlFlow::Exit;
					return;
				}
				ccmd::Request::None => {}
			}
		}

		self.audio.update();
	}

	pub fn on_window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.gfx.resize(new_size);
		self.camera
			.resize(new_size.width as f32, new_size.height as f32);
	}

	pub fn on_key_event(&mut self, event: &KeyboardInput) {
		self.console.on_key_event(event);
		self.input.on_key_event(event);

		if event.virtual_keycode.is_none() {
			return;
		} else if let Some(VirtualKeyCode::Grave) = event.virtual_keycode {
			if event.state == ElementState::Pressed {
				self.devgui.open = !self.devgui.open;
			}
		}

		let vkc = event.virtual_keycode.unwrap();
		let _binds = self
			.input
			.user_binds
			.iter()
			.filter(|kb| kb.keycode == vkc && kb.modifiers == self.input.modifiers);

		// TODO: Invoke LithScript callbacks.
	}
}
