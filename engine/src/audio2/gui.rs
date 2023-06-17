use bevy_egui::egui;
use indoc::formatdoc;
use kira0_8_3::sound::PlaybackState;

use crate::data::Catalog;

use super::AudioCore;

impl AudioCore {
	pub(super) fn ui_impl(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _: &Catalog) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			// Music ///////////////////////////////////////////////////////////

			ui.heading("Music");

			ui.separator();

			// Sounds //////////////////////////////////////////////////////////

			ui.heading("Sounds");

			ui.label(&self.ui_label_song_counters());

			ui.separator();

			// Quick play //////////////////////////////////////////////////////

			ui.heading("Quick Play");

			ui.add(
				egui::Slider::new(&mut self.gui.volume, 0.0..=4.0)
					.text("Volume")
					.custom_formatter(|val, _| format!("{:03.2}%", val * 100.0)),
			);

			ui.label("(VFS Path/Data ID)");

			ui.horizontal(|ui| {
				ui.text_edit_singleline(&mut self.gui.id_buf);
			});

			ui.separator();

			// Diagnostics /////////////////////////////////////////////////////

			ui.heading("Diagnostics");

			ui.label(formatdoc! {"
			Sound capacity: {snd_cap}
			Mixer sub-track capacity: {track_cap}
			Clock capacity: {clock_cap}",
			snd_cap = self.manager.sound_capacity(),
			track_cap = self.manager.num_sub_tracks(),
			clock_cap = self.manager.clock_capacity(),
			});
		});
	}

	#[must_use]
	fn ui_label_song_counters(&self) -> String {
		let mut playing = 0;
		let mut paused = 0;

		for channel in self.sounds.iter() {
			let Some(sfx) = channel else { continue; };

			if sfx.state() == PlaybackState::Playing {
				playing += 1;
			}

			if sfx.state() == PlaybackState::Paused {
				paused += 1;
			}
		}

		format!(
			"{playing} playing, {paused} paused, {changing} changing, {total} total",
			changing = self.sounds.len() - (playing + paused),
			total = self.sounds.len(),
		)
	}
}

/// State storage for the audio developer GUI.
#[derive(Debug)]
pub(super) struct DevGui {
	/// Let the user write a VFS path or data object ID.
	pub(super) id_buf: String,
	/// For allowing the user to enter a custom SoundFont path.
	///
	/// Set by default from the first SoundFont collected during audio core
	/// initialization. If none were found, this will be empty.
	pub(super) soundfont_buf: String,
	/// Amplitude. Slider runs from 0.0 to 4.0.
	pub(super) volume: f64,
}

impl Default for DevGui {
	fn default() -> Self {
		Self {
			id_buf: String::new(),
			soundfont_buf: String::new(),
			volume: 1.0,
		}
	}
}
