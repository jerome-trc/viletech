//! Functions for reading data objects from WADs.

use std::{io::Cursor, ops::Range};

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use rayon::prelude::*;

use crate::{
	data::{dobj::Audio, Catalog, PrepError, PrepErrorKind},
	vfs::FileRef,
	Outcome,
};

use super::SubContext;

#[derive(Debug)]
struct Markers {
	flats: Option<Range<usize>>,
	sprites: Option<Range<usize>>,
}

impl Markers {
	#[must_use]
	fn new(wad: FileRef) -> Self {
		let mut buf = wad.path().join("F_START");

		Markers {
			flats: {
				if let Some(f_start) = wad.child_index(&buf) {
					buf.pop();
					buf.push("F_END");

					wad.child_index(&buf).map(|f_end| (f_start + 1)..f_end)
				} else {
					None
				}
			},
			sprites: {
				buf.pop();
				buf.push("S_START");

				if let Some(s_start) = wad.child_index(&buf) {
					buf.pop();
					buf.push("S_END");

					wad.child_index(&buf).map(|s_end| (s_start + 1)..s_end)
				} else {
					None
				}
			},
		}
	}

	#[must_use]
	fn is_flat(&self, child_index: usize) -> bool {
		if let Some(flats) = &self.flats {
			return flats.contains(&child_index);
		}

		false
	}

	#[must_use]
	fn is_sprite(&self, child_index: usize) -> bool {
		if let Some(sprites) = &self.sprites {
			return sprites.contains(&child_index);
		}

		false
	}
}

impl Catalog {
	pub(super) fn prep_pass1_wad(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}

	pub(super) fn prep_pass2_wad(&self, ctx: &SubContext) {
		let wad = self.vfs.get(ctx.mntinfo.mount_point()).unwrap();

		wad.children().unwrap().par_bridge().try_for_each(|child| {
			if !child.is_readable() {
				return Some(());
			}

			if ctx.is_cancelled() {
				return None;
			}

			let bytes = child.read_bytes();
			let fstem = child.file_prefix();

			if Audio::is_flac(bytes)
				|| Audio::is_mp3(bytes)
				|| Audio::is_ogg(bytes)
				|| Audio::is_wav(bytes)
			{
				let cursor = Cursor::new(bytes.to_owned());

				match StaticSoundData::from_cursor(cursor, StaticSoundSettings::default()) {
					Ok(statsnd) => {
						ctx.add_datum(Audio::Waveform(statsnd), child.file_prefix());
					}
					Err(err) => {
						ctx.raise_error(PrepError {
							path: child.path().to_path_buf(),
							kind: PrepErrorKind::WaveformAudio(err),
						});
					}
				}

				return Some(());
			}

			if fstem == "COLORMAP" {
				match self.prep_colormap(child, bytes) {
					Ok(colormap) => {
						ctx.arts_w.lock().colormap = Some(Box::new(colormap));
					}
					Err(err) => ctx.raise_error(*err),
				}

				return Some(());
			}

			if fstem == "ENDOOM" {
				match self.prep_endoom(child, bytes) {
					Ok(endoom) => ctx.arts_w.lock().endoom = Some(Box::new(endoom)),
					Err(err) => ctx.raise_error(*err),
				}
			}

			if fstem == "PLAYPAL" {
				match self.prep_playpal(ctx, bytes) {
					Ok(palset) => {
						ctx.arts_w.lock().palset = Some(Box::new(palset));
					}
					Err(err) => {
						ctx.raise_error(PrepError {
							path: child.path().to_path_buf(),
							kind: PrepErrorKind::Io(err),
						});
					}
				}

				return Some(());
			}

			if fstem == "PNAMES" {
				match self.prep_pnames(ctx, child, bytes) {
					Outcome::Ok(mut pnames) => {
						ctx.arts_w.lock().pnames.append(&mut pnames);
					}
					Outcome::Err(err) => ctx.raise_error(err),
					Outcome::None => {}
					_ => unreachable!(),
				}

				return Some(());
			}

			if fstem == "TEXTURE1"
				|| fstem == "TEXTURE2"
				|| fstem == "TEXTURE3"
				|| fstem == "TEXTURES"
			{
				match self.prep_texturex(ctx, child, bytes) {
					Outcome::Ok(mut texx) => {
						ctx.arts_w.lock().texturex.append(&mut texx);
					}
					Outcome::Err(err) => ctx.raise_error(err),
					Outcome::None => {}
					_ => unreachable!(),
				}
			}

			Some(())
		});
	}

	pub(super) fn prep_pass3_wad(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let wad = self.vfs.get(ctx.mntinfo.mount_point()).unwrap();
		let markers = Markers::new(wad);

		let proceed = wad
			.children()
			.unwrap()
			.filter(|c| !c.is_empty())
			.enumerate()
			.par_bridge()
			.try_for_each(|(cndx, child)| {
				if ctx.is_cancelled() {
					return None;
				}

				if child.is_dir() {
					self.prep_pass3_wad_dir(ctx, child);
				} else {
					self.prep_pass3_wad_entry(ctx, &markers, child, cndx);
				};

				Some(())
			});

		match proceed {
			Some(()) => Outcome::Ok(()),
			None => Outcome::Cancelled,
		}
	}

	fn prep_pass3_wad_entry(
		&self,
		ctx: &SubContext,
		markers: &Markers,
		vfile: FileRef,
		child_index: usize,
	) {
		let bytes = vfile.read_bytes();
		let fpfx = vfile.file_prefix();

		ctx.higher.tracker.add_to_progress(1);

		if markers.is_flat(child_index) {
			match self.prep_flat(ctx, vfile, bytes) {
				Ok(image) => ctx.add_datum(image, fpfx),
				Err(err) => ctx.raise_error(*err),
			}

			return;
		}

		if markers.is_sprite(child_index) {
			match self.prep_picture(ctx, bytes) {
				Some(image) => ctx.add_datum(image, fpfx),
				None => {
					ctx.raise_error(PrepError {
						path: vfile.path().to_path_buf(),
						kind: PrepErrorKind::Sprite,
					});
				}
			}

			return;
		}

		if fpfx.starts_with("DEMO") && fpfx.ends_with(|c| char::is_ascii_digit(&c)) {
			// TODO: Demo file format support.
			return;
		}

		/// Kinds of WAD entries irrelevant to this pass.
		const UNHANDLED: &[&str] = &[
			"COLORMAP", "DMXGUS", "ENDOOM", "GENMIDI", "PLAYPAL", "PNAMES", "TEXTURE1", "TEXTURE2",
			"TEXTURE3", "TEXTURES",
		];

		if UNHANDLED.iter().any(|&name| fpfx == name)
			|| Audio::is_pc_speaker_sound(bytes)
			|| Audio::is_dmxmus(bytes)
		{
			return;
		}

		if let Some(image) = self.prep_picture(ctx, bytes) {
			ctx.add_datum(image, fpfx);
		}

		// Else this file has an unknown purpose.
		// User scripts may have their own intent for it, so this is fine.
	}

	fn prep_pass3_wad_dir(&self, ctx: &SubContext, dir: FileRef) {
		match self.try_prep_level_vanilla(ctx, dir) {
			Outcome::Ok(level) => {
				ctx.add_datum(level, dir.file_prefix());
			}
			Outcome::Err(()) => return,
			Outcome::None => {}
			_ => unreachable!(),
		}

		match self.try_prep_level_udmf(ctx, dir) {
			Outcome::Ok(level) => {
				ctx.add_datum(level, dir.file_prefix());
			}
			Outcome::Err(()) => {}
			Outcome::None => {}
			_ => unreachable!(),
		}
	}
}
