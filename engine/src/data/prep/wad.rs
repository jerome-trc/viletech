//! Functions for reading assets from WADs.

use std::io::Cursor;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use rayon::prelude::*;

use crate::data::{
	detail::Outcome, vfs::FileRef, AssetHeader, Audio, AudioData, Catalog, PrepError, PrepErrorKind,
};

use super::SubContext;

impl Catalog {
	pub(super) fn prep_pass1_wad(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}

	pub(super) fn prep_pass2_wad(&self, ctx: &SubContext) {
		let wad = self.vfs.get(ctx.mntinfo.virtual_path()).unwrap();

		wad.children().unwrap().par_bridge().try_for_each(|child| {
			if !child.is_readable() {
				return Some(());
			}

			if ctx.tracker.is_cancelled() {
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
						ctx.add_asset(Audio {
							header: AssetHeader {
								id: format!(
									"{mount_id}/{id}",
									mount_id = ctx.mntinfo.id(),
									id = child.file_stem()
								),
							},
							data: AudioData::Waveform(statsnd),
						});
					}
					Err(err) => {
						ctx.errors.lock().push(PrepError {
							path: child.path.to_path_buf(),
							kind: PrepErrorKind::WaveformAudio(err),
							fatal: false,
						});
					}
				}

				return Some(());
			}

			if fstem == "COLORMAP" {
				match self.prep_colormap(
					FileRef {
						vfs: &self.vfs,
						file: child,
					},
					bytes,
				) {
					Ok(colormap) => {
						ctx.artifacts.lock().extras.colormap = Some(Box::new(colormap));
					}
					Err(err) => ctx.errors.lock().push(*err),
				}

				return Some(());
			}

			if fstem == "ENDOOM" {
				match self.prep_endoom(
					FileRef {
						vfs: &self.vfs,
						file: child,
					},
					bytes,
				) {
					Ok(endoom) => ctx.artifacts.lock().extras.endoom = Some(Box::new(endoom)),
					Err(err) => ctx.errors.lock().push(*err),
				}
			}

			if fstem == "PLAYPAL" {
				match self.prep_playpal(ctx, bytes) {
					Ok(palset) => {
						ctx.artifacts.lock().extras.palset = Some(Box::new(palset));
					}
					Err(err) => {
						ctx.errors.lock().push(PrepError {
							path: child.path().to_path_buf(),
							kind: PrepErrorKind::Io(err),
							fatal: false,
						});
					}
				}

				return Some(());
			}

			if fstem == "PNAMES" {
				match self.prep_pnames(
					ctx,
					FileRef {
						vfs: &self.vfs,
						file: child,
					},
					bytes,
				) {
					Outcome::Ok(pnames) => ctx.artifacts.lock().pnames = Some(pnames),
					Outcome::Err(err) => ctx.errors.lock().push(err),
					Outcome::None => {}
					_ => unreachable!(),
				}

				return Some(());
			}

			if fstem == "TEXTURE1" || fstem == "TEXTURE2" {
				match self.prep_texturex(
					ctx,
					FileRef {
						vfs: &self.vfs,
						file: child,
					},
					bytes,
				) {
					Outcome::Ok(texturex) => {
						if fstem.ends_with('1') {
							ctx.artifacts.lock().texture1 = Some(texturex);
						} else if fstem.ends_with('2') {
							ctx.artifacts.lock().texture2 = Some(texturex);
						}
					}
					Outcome::Err(err) => ctx.errors.lock().push(err),
					Outcome::None => {}
					_ => unreachable!(),
				}
			}

			Some(())
		});
	}

	pub(super) fn prep_pass3_wad(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let wad = self.vfs.get(ctx.mntinfo.virtual_path()).unwrap();

		let cancelled = wad
			.child_refs()
			.unwrap()
			.filter(|c| !c.is_empty())
			.par_bridge()
			.try_for_each(|child| {
				if ctx.tracker.is_cancelled() {
					return None;
				}

				if child.is_dir() {
					self.prep_pass3_wad_dir(ctx, child);
				} else {
					self.prep_pass3_wad_entry(ctx, child);
				};

				Some(())
			});

		match cancelled {
			Some(()) => Outcome::Ok(()),
			None => Outcome::Cancelled,
		}
	}

	fn prep_pass3_wad_entry(&self, ctx: &SubContext, vfile: FileRef) {
		let bytes = vfile.read_bytes();
		let fstem = vfile.file_prefix();

		/// Kinds of WAD entries irrelevant to this pass.
		const UNHANDLED: &[&str] = &[
			"COLORMAP", "DMXGUS", "ENDOOM", "GENMIDI", "PLAYPAL", "PNAMES", "TEXTURE1", "TEXTURE2",
		];

		if UNHANDLED.iter().any(|&name| fstem == name)
			|| Audio::is_pc_speaker_sound(bytes)
			|| Audio::is_dmxmus(bytes)
		{
			return;
		}

		ctx.tracker.add_prep_progress(1);

		let is_pic = self.prep_picture(ctx, bytes, fstem);

		// TODO: Processors for more file formats.

		let res: std::io::Result<()> = if is_pic.is_some() {
			Ok(())
		} else {
			return;
		};

		match res {
			Ok(()) => {}
			Err(err) => {
				ctx.errors.lock().push(PrepError {
					path: vfile.path().to_path_buf(),
					kind: PrepErrorKind::Io(err),
					fatal: true,
				});
			}
		}
	}

	fn prep_pass3_wad_dir(&self, ctx: &SubContext, dir: FileRef) {
		match self.try_prep_level_vanilla(ctx, dir) {
			Outcome::Ok(()) | Outcome::Err(()) => return,
			Outcome::None => {}
			_ => unreachable!(),
		}

		match self.try_prep_level_udmf(ctx, dir) {
			Outcome::Ok(()) | Outcome::Err(()) => {}
			Outcome::None => {}
			_ => unreachable!(),
		}
	}
}
