//! Internal data preparation functions.
//!
//! After mounting is done, start composing useful objects from raw files.

mod level;
mod pk37;
mod udmf;
mod vanilla;
mod wad;

use std::sync::Arc;

use bevy::prelude::{info, warn};
use dashmap::DashMap;
use data::gfx::{ColorMap, EnDoom, PaletteSet, PatchTable, TextureX};
use parking_lot::Mutex;
use rayon::prelude::*;
use serde::Deserialize;
use smallvec::{smallvec, SmallVec};
use util::{EditorNum, Outcome, SendTracker, SpawnNum};
use vfs::VPathBuf;

use crate::{catalog::dobj::DATUM_TYPE_NAMES, vfs::MountFormat};

use super::{
	dobj::{DatumStore, Store},
	Catalog, Datum, DatumKey, MountInfo, PrepError, PrepErrorKind,
};

type Output = Vec<Vec<PrepError>>;

impl Catalog {
	/// Preconditions:
	/// - `self.vfs` has been populated. All directories know their contents.
	/// - `ctx.tracker` has already had its target number set.
	pub(super) fn prep(&mut self, mut ctx: Context) -> Outcome<Output, Output> {
		// Pass 1: determine how each mount needs to be processed.
		// Compile VZS; transpile EDF and (G)ZDoom DSLs.

		for (i, mount) in self.vfs.mounts().iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			ctx.arts[i].kind = self.resolve_mount_kind(mount);
			self.resolve_mount_metadata(mount, &mut ctx.arts[i]);

			let subctx = SubContext {
				higher: &ctx,
				mntinfo: mount,
				arts: &ctx.arts[i],
				arts_w: &ctx.arts_working[i],
			};

			let _ = match ctx.arts[i].kind {
				MountKind::VileTech => self.prep_pass1_vpk(&subctx),
				MountKind::ZDoom => self.prep_pass1_pk(&subctx),
				MountKind::Eternity => todo!(),
				MountKind::Wad => self.prep_pass1_wad(&subctx),
				MountKind::Misc => self.prep_pass1_file(&subctx),
			};
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish();
			return Outcome::Err(ctx.into_errors());
		}

		// Pass 2: dependency-free assets; trivial to parallelize. Includes:
		// - Palettes and colormaps.
		// - ENDOOM.
		// - TEXTUREX and PNAMES.
		// - Sounds and music.
		// - Non-picture-format images.

		for (i, mount) in self.vfs.mounts().iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				higher: &ctx,
				mntinfo: mount,
				arts: &ctx.arts[i],
				arts_w: &ctx.arts_working[i],
			};

			match ctx.arts[i].kind {
				MountKind::Wad => self.prep_pass2_wad(&subctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish();
			return Outcome::Err(ctx.into_errors());
		}

		ctx.post_pass2();

		if ctx.last_paletteset().is_none() {
			unimplemented!("further loading without a PLAYPAL is unsupported for now");
		}

		// Pass 3: assets dependent on pass 2. Includes:
		// - Picture-format images, which need palettes and PNAMES.
		// - Maps, which need textures, music, scripts, blueprints...

		for (i, mount) in self.vfs.mounts().iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				higher: &ctx,
				mntinfo: mount,
				arts: &ctx.arts[i],
				arts_w: &ctx.arts_working[i],
			};

			let _ = match ctx.arts[i].kind {
				MountKind::Wad => self.prep_pass3_wad(&subctx),
				MountKind::VileTech => Outcome::None, // Soon!
				_ => unimplemented!("Soon!"),
			};
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish();
			return Outcome::Err(ctx.into_errors());
		}

		ctx.post_pass3();

		let Context {
			tracker,
			dobjs,
			nicknames,
			editor_nums,
			spawn_nums,
			arts_working,
			arts: _,
		} = ctx;

		self.dobjs = dobjs.into_read_only();
		self.nicknames = nicknames.into_read_only();
		self.editor_nums = editor_nums.into_read_only();
		self.spawn_nums = spawn_nums.into_read_only();

		info!("Loading complete.");

		// TODO: Make each successfully processed file increment progress.
		tracker.finish();

		Outcome::Ok(Context::rollup_errors(arts_working))
	}

	/// Try to compile non-ACS scripts from this package. VZS, EDF, and (G)ZDoom
	/// DSLs all go into the same VZS library, regardless of which are present
	/// and which are absent.
	fn prep_pass1_vpk(&self, ctx: &SubContext) -> Outcome<(), ()> {
		if let Some(vzscript) = &ctx.arts.vzscript {
			let root_dir_path: VPathBuf = [ctx.mntinfo.mount_point(), &vzscript.root_dir]
				.iter()
				.collect();

			let _ = match self.vfs.get(&root_dir_path) {
				Some(fref) => fref,
				None => {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(&vzscript.root_dir),
						kind: PrepErrorKind::MissingVzsDir,
					});

					return Outcome::Err(());
				}
			};

			if ctx.is_cancelled() {
				return Outcome::Cancelled;
			}

			/*

			TODO: Soon!

			let mut inctree = vzs::IncludeTree::new(root_dir);

			if inctree.any_errors() {
				let errors = &mut ctx.arts_w.lock().errors;
				let parse_errs = inctree.drain_errors();

				for (path, err) in parse_errs {
					errors.push(PrepError {
						path: path.clone(),
						kind: PrepErrorKind::VzsParse(err),
					});
				}
			}

			*/

			if ctx.is_cancelled() {
				return Outcome::Cancelled;
			}
		}

		Outcome::None
	}

	fn prep_pass1_file(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let file = self.vfs.get(ctx.mntinfo.mount_point()).unwrap();

		// Pass 1 only deals in text files.
		if !file.is_text() {
			return Outcome::None;
		}

		if ctx.is_cancelled() {
			return Outcome::Cancelled;
		}

		if file
			.path_extension()
			.filter(|p_ext| p_ext.eq_ignore_ascii_case("vzs"))
			.is_some()
		{
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("decorate") {
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("zscript") {
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("edfroot") {
			unimplemented!();
		}

		Outcome::None
	}

	// Details /////////////////////////////////////////////////////////////////

	/// Assumes that `self.vfs` has been fully populated.
	#[must_use]
	fn resolve_mount_kind(&self, mount: &MountInfo) -> MountKind {
		if mount.format() == MountFormat::Wad {
			return MountKind::Wad;
		}

		let fref = self
			.vfs
			.get(mount.mount_point())
			.expect("`resolve_mount_kind` received an invalid virtual path");

		if fref.is_leaf() {
			return MountKind::Misc;
		}

		// Heuristics have a precedence hierarchy, so use multiple passes.

		if fref
			.children()
			.unwrap()
			.any(|child| child.file_name().eq_ignore_ascii_case("meta.toml") && child.is_text())
		{
			return MountKind::VileTech;
		}

		const ZDOOM_FILE_PFXES: &[&str] = &[
			"cvarinfo", "decorate", "gldefs", "menudef", "modeldef", "sndinfo", "zmapinfo",
			"zscript",
		];

		if fref.children().unwrap().any(|child| {
			let pfx = child.file_prefix();

			ZDOOM_FILE_PFXES
				.iter()
				.any(|&constant| pfx.eq_ignore_ascii_case(constant))
		}) {
			return MountKind::ZDoom;
		}

		if fref.children().unwrap().any(|child| {
			let fstem = child.file_prefix();
			fstem.eq_ignore_ascii_case("edfroot") || fstem.eq_ignore_ascii_case("emapinfo")
		}) {
			return MountKind::Eternity;
		}

		unreachable!("all mount kind resolution heuristics failed")
	}

	/// Parses a meta.toml if one exists. Otherwise, make a best-possible effort
	/// to deduce some metadata. Assumes that `self.vfs` has been fully populated,
	/// and that `arts` already knows its kind.
	fn resolve_mount_metadata(&self, info: &MountInfo, arts: &mut Artifacts) {
		debug_assert!(!info.id().is_empty());

		if arts.kind != MountKind::VileTech {
			// Q: Should we bother trying to infer the mount's version?
			return;
		}

		let meta_path = info.mount_point().join("meta.toml");
		let meta_file = self.vfs.get(&meta_path).unwrap();

		let ingest: MountMetaIngest = match toml::from_str(meta_file.read_str()) {
			Ok(toml) => toml,
			Err(err) => {
				warn!(
					"Invalid meta.toml file: {p}\r\n\t\
					Details: {err}\r\n\t\
					This mount's metadata may be incomplete.",
					p = meta_path.display()
				);

				return;
			}
		};

		if let Some(mnf) = ingest.vzscript {
			let version = match mnf.version.parse::<vzs::Version>() {
				Ok(v) => v,
				Err(err) => {
					warn!(
						"Invalid `vzscript` table in meta.toml file: {p}\r\n\t\
						Details: {err}\r\n\t\
						This mount's metadata may be incomplete.",
						p = meta_path.display()
					);

					return;
				}
			};

			arts.vzscript = Some(VzsManifest {
				root_dir: mnf.folder,
				_namespace: mnf.namespace,
				_version: version,
			});
		}
	}
}

// Intermediate structures /////////////////////////////////////////////////////

#[derive(Debug)]
pub(super) struct Context {
	tracker: Arc<SendTracker>,
	dobjs: DashMap<DatumKey, Arc<dyn DatumStore>>,
	nicknames: DashMap<DatumKey, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	editor_nums: DashMap<EditorNum, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	spawn_nums: DashMap<SpawnNum, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	arts_working: Vec<Mutex<WorkingArtifacts>>,
	arts: Vec<Artifacts>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Arc<SendTracker>, mounts_len: usize) -> Self {
		debug_assert!(tracker.target() != 0);

		let dobjs = DashMap::with_capacity(tracker.target());

		Self {
			tracker,
			dobjs,
			nicknames: DashMap::default(),
			editor_nums: DashMap::default(),
			spawn_nums: DashMap::default(),
			arts_working: {
				let mut a = vec![];
				a.resize_with(mounts_len, || Mutex::new(WorkingArtifacts::default()));
				a
			},
			arts: {
				let mut a = vec![];
				a.resize_with(mounts_len, Artifacts::default);
				a
			},
		}
	}

	#[must_use]
	pub(super) fn any_fatal_errors(&self) -> bool {
		self.arts_working
			.par_iter()
			.any(|mutex| mutex.lock().errors.iter().any(|err| err.is_fatal()))
	}

	#[must_use]
	fn rollup_errors(arts_working: Vec<Mutex<WorkingArtifacts>>) -> Vec<Vec<PrepError>> {
		arts_working
			.into_iter()
			.map(|mutex| mutex.into_inner().errors)
			.collect()
	}

	#[must_use]
	fn into_errors(self) -> Vec<Vec<PrepError>> {
		Self::rollup_errors(self.arts_working)
	}

	#[must_use]
	fn last_paletteset(&self) -> Option<&PaletteSet> {
		self.arts.iter().filter_map(|a| a.palset.as_deref()).last()
	}

	fn post_pass2(&mut self) {
		for (w, a) in self.arts_working.iter().zip(self.arts.iter_mut()) {
			let mut w = w.lock();

			a.texturex = std::mem::take(&mut w.texturex);
			a.pnames = std::mem::take(&mut w.pnames);
			a.endoom = std::mem::take(&mut w.endoom);
			a.palset = std::mem::take(&mut w.palset);
			a.colormap = std::mem::take(&mut w.colormap);
		}
	}

	fn post_pass3(&mut self) {
		// ???
	}
}

/// Read-only prep artifacts that don't need to be behind a mutex.
/// Associated with one mount. All get discarded when prep finishes.
#[derive(Debug, Default)]
struct Artifacts {
	kind: MountKind,
	vzscript: Option<VzsManifest>,
	texturex: TextureX,
	pnames: PatchTable,
	colormap: Option<Box<ColorMap>>,
	palset: Option<Box<PaletteSet>>,
	endoom: Option<Box<EnDoom>>,
}

/// Working buffer for prep artifacts to put behind a mutex.
/// Associated with one mount.
#[derive(Debug, Default)]
struct WorkingArtifacts {
	/// Preserved between passes; only discharged when prep finishes.
	errors: Vec<PrepError>,
	colormap: Option<Box<ColorMap>>,
	palset: Option<Box<PaletteSet>>,
	endoom: Option<Box<EnDoom>>,
	texturex: TextureX,
	pnames: PatchTable,
}

#[derive(Debug)]
struct VzsManifest {
	/// The base of the package's VZScript include tree.
	///
	/// This is irrelevant to WADs, which can only use VZS through lumps named
	/// `VZSCRIPT`.
	///
	/// Normally, the scripts can define manifest items used to direct loading,
	/// but if there is no script root or manifests, ZDoom loading rules are used.
	///
	/// A package can only specify a directory owned by it as a script root, so this
	/// is always relative. `viletech.vpk3`'s script root, for example, is `scripts`.
	pub(super) root_dir: VPathBuf,
	pub(super) _namespace: Option<String>,
	pub(super) _version: vzs::Version,
}

/// Context relevant to operations on one mount.
#[derive(Debug)]
struct SubContext<'ctx> {
	higher: &'ctx Context,
	mntinfo: &'ctx MountInfo,
	arts: &'ctx Artifacts,
	arts_w: &'ctx Mutex<WorkingArtifacts>,
}

/// Informs the rules used for preparing data from a mount.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum MountKind {
	/// If the mount's own root has an immediate child text file named `meta.toml`
	/// (ASCII-case-ignored), that indicates that the mount is a VileTech package.
	VileTech,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`, and
	/// - the extension `.pk3`, `.ipk3`, `.pk7`, or `.ipk7`,
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with a file
	/// stem belonging to a ZDoom-exclusive lump.
	ZDoom,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`, and
	/// - the extension `.pke`,
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with the
	/// file stem `edfroot` or `emapinfo` (ASCII-case-ignored).
	Eternity,
	/// Deduced from [`MountFormat`], which is itself deduced from the file header.
	Wad,
	/// Fallback if the mount resolved to none of the other kinds.
	/// Usually used if mounting a single non-archive file.
	#[default]
	Misc,
}

impl SubContext<'_> {
	fn add_datum<D: Datum>(&self, datum: D, id_suffix: impl AsRef<str>) {
		let id = format!("{}/{}", self.mntinfo.id(), id_suffix.as_ref());

		let key = DatumKey::new::<D>(&id);
		let key_nick = DatumKey::new::<D>(id.split('/').last().unwrap());

		let store: Arc<dyn DatumStore> = Arc::new(Store::new(id, datum));

		match self.higher.dobjs.entry(key) {
			dashmap::mapref::entry::Entry::Occupied(mut occu) => {
				info!(
					"Overwriting: {} ({})",
					store.id(),
					DATUM_TYPE_NAMES.get(&store.type_id()).unwrap(),
				);

				occu.insert(store.clone());
			}
			dashmap::mapref::entry::Entry::Vacant(vacant) => {
				vacant.insert(store.clone());
			}
		}

		if let Some(mut kvp) = self.higher.nicknames.get_mut(&key_nick) {
			kvp.value_mut().push(store);
		} else {
			self.higher.nicknames.insert(key_nick, smallvec![store]);
		};
	}

	fn raise_error(&self, err: PrepError) {
		self.arts_w.lock().errors.push(err);
	}

	#[must_use]
	fn is_cancelled(&self) -> bool {
		self.higher.tracker.is_cancelled()
	}
}

/// Intermediate format for parsing parts of [`MountMeta`] from `meta.toml` files.
///
/// [`MountMeta`]: super::MountMeta
#[derive(Debug, Default, Deserialize)]
#[allow(unused, dead_code)] // TODO: Revisit all of this.
pub(super) struct MountMetaIngest {
	pub id: String,
	#[serde(default)]
	pub version: Option<String>,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub description: Option<String>,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub copyright: Option<String>,
	#[serde(default)]
	pub links: Vec<String>,
	#[serde(default)]
	pub vzscript: Option<MountMetaIngestVzs>,
}

#[derive(Debug, Deserialize)]
pub(super) struct MountMetaIngestVzs {
	pub folder: VPathBuf,
	pub namespace: Option<String>,
	pub version: String,
}
