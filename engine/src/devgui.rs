use bevy_egui::egui;

/// State and functions for a two-panel egui window that sticks to the top of the
/// screen like GZDoom's console. `S` should be a simple untagged enum that
/// informs the user what they should draw in each panel.
#[derive(Debug)]
pub struct DeveloperGui<S: PartialEq + Copy> {
	pub open: bool,
	pub left: S,
	pub right: S,
}

impl<S: PartialEq + Copy> DeveloperGui<S> {
	/// Returns an egui window that:
	/// - Is 80% opaque
	/// - Stretches to fill the screen's width
	/// - Is immovably anchored to the screen's top
	/// - Can be resized, but only vertically
	pub fn window(ctx: &egui::Context) -> egui::containers::Window {
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		egui::Window::new("Developer Tools")
			.id(egui::Id::new("vile_devgui"))
			.anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
			.fixed_pos([0.0, 0.0])
			.collapsible(false)
			.resizable(true)
			.min_width(screen_rect.width())
			.min_height(screen_rect.height() * 0.1)
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	pub fn panel_left(&self, ctx: &egui::Context) -> egui::SidePanel {
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		egui::SidePanel::left("vile_devgui_left")
			.default_width(screen_rect.width() * 0.5)
			.resizable(true)
			.width_range((screen_rect.width() * 0.1)..=(screen_rect.width() * 0.9))
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	/// Ensure this is only called after [`panel_left`](DeveloperGui::panel_left).
	pub fn panel_right(&self, ctx: &egui::Context) -> egui::CentralPanel {
		egui::CentralPanel::default()
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	/// Call after opening the [developer GUI window](DeveloperGui::window).
	/// Draws two dropdowns in its menu bar that allow changing which menu is
	/// being drawn in each pane. A menu can't replace itself, but the left and
	/// right side can be swapped.
	pub fn selectors(&mut self, ui: &mut egui::Ui, choices: &[(S, &str)]) {
		egui::menu::bar(ui, |ui| {
			ui.menu_button("Left", |ui| {
				for (choice, label) in choices {
					let btn = egui::Button::new(*label);
					let resp = ui.add_enabled(self.left != *choice, btn);

					if resp.clicked() {
						ui.close_menu();

						if self.right == *choice {
							std::mem::swap(&mut self.left, &mut self.right);
						} else {
							self.left = *choice;
						}
					}
				}
			});

			ui.menu_button("Right", |ui| {
				for (choice, label) in choices {
					let btn = egui::Button::new(*label);
					let resp = ui.add_enabled(self.right != *choice, btn);

					if resp.clicked() {
						ui.close_menu();

						if self.left == *choice {
							std::mem::swap(&mut self.left, &mut self.right);
						} else {
							self.right = *choice;
						}
					}
				}
			});
		});
	}
}
