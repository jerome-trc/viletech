//! Internal implementation details related to mounting and unmounting files.
//!
//! Step 1 of a game load is building the virtual file system.

use bevy::prelude::warn;

use crate::{
	data::{detail::MountMetaIngest, MountInfo, MountKind, MountMeta, VzsManifest},
	vzs, VPath,
};

use super::{Catalog, MountFormat};

impl Catalog {
	/// Assumes that `self.vfs` has been fully populated.
	#[must_use]
	pub(super) fn resolve_mount_kind(
		&self,
		format: MountFormat,
		virtual_path: impl AsRef<VPath>,
	) -> MountKind {
		if format == MountFormat::Wad {
			return MountKind::Wad;
		}

		let fref = self
			.vfs
			.get(virtual_path)
			.expect("`resolve_mount_kind` received an invalid virtual path.");

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

		unreachable!("All mount kind resolution heuristics failed.")
	}

	/// Parses a meta.toml if one exists. Otherwise, make a best-possible effort
	/// to deduce some metadata. Assumes that `self.files` has been fully populated.
	pub(super) fn resolve_mount_metadata(&self, info: &mut MountInfo) {
		debug_assert!(!info.id.is_empty());

		if info.kind != MountKind::VileTech {
			// Q: Should we bother trying to infer the mount's version?
			return;
		}

		let meta_path = info.virtual_path().join("meta.toml");
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

		info.id = ingest.id;

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

			info.vzscript = Some(VzsManifest {
				root_dir: mnf.folder,
				namespace: mnf.namespace,
				version,
			});
		}

		info.meta = Some(Box::new(MountMeta {
			version: ingest.version,
			name: ingest.name,
			description: ingest.description,
			authors: ingest.authors,
			copyright: ingest.copyright,
			links: ingest.links,
		}));
	}
}
