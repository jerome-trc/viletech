//! VileTech's [`egui`]-based two-pane developer graphical user interface.

use bevy::prelude::Resource;
use bevy_egui::egui;

/// State and functions for a two-panel egui window that sticks to the top of the
/// screen like GZDoom's console. `S` should be a simple untagged enum that
/// informs the user what they should draw in each panel.
#[derive(Debug, Resource)]
pub struct DeveloperGui<S>
where
	S: Eq + Copy + std::fmt::Display,
{
	pub open: bool,
	pub left: S,
	pub right: S,
}

impl<S> DeveloperGui<S>
where
	S: Eq + Copy + std::fmt::Display,
{
	/// Returns an egui window that:
	/// - Is 80% opaque
	/// - Stretches to fill the screen's width
	/// - Is immovably anchored to the screen's top
	/// - Can be resized, but only vertically
	pub fn window(ctx: &egui::Context) -> egui::containers::Window {
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		egui::Window::new("Developer Tools")
			.id(egui::Id::new("viletech_dgui"))
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

		egui::SidePanel::left("viletech_dgui_left")
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

	/// Call after opening the [developer GUI window](DeveloperGui::window) and
	/// then calling [`egui::menu::bar`]. Draws two dropdowns that allow changing
	/// which menu is being drawn in each pane. A menu cannot replace itself, but
	/// the left and right side can be swapped.
	pub fn selectors(&mut self, ui: &mut egui::Ui, choices: &[(S, &str)]) {
		egui::ComboBox::new("viletech_dgui_selector_left", "Left")
			.selected_text(format!("{}", self.left))
			.show_ui(ui, |ui| {
				let cur = self.left;

				for (choice, label) in choices.iter().copied() {
					ui.selectable_value(&mut self.left, choice, label);
				}

				if self.right == self.left {
					self.right = cur;
				}
			});

		ui.separator();

		egui::ComboBox::new("viletech_dgui_selector_right", "Right")
			.selected_text(format!("{}", self.right))
			.show_ui(ui, |ui| {
				let cur = self.right;

				for (choice, label) in choices.iter().copied() {
					ui.selectable_value(&mut self.right, choice, label);
				}

				if self.left == self.right {
					self.left = cur;
				}
			});
	}
}
