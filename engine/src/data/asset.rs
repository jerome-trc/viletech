//! A trait providing generic functionality for [`super::DataCore`].

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	fmt,
	ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::replace_expr;

use super::{AssetVec, Namespace};

pub trait Asset: Sized {
	/// Wherever a homogenous array of items is declared wherein there must be
	/// one element per asset type, this constant is used to index into it.
	const INDEX: usize;

	/// Should refer to one of the members of [`Namespace`].
	#[must_use]
	fn collection(namespace: &Namespace) -> &AssetVec<Self>;
	/// Should refer to one of the members of [`Namespace`].
	#[must_use]
	fn collection_mut(namespace: &mut Namespace) -> &mut AssetVec<Self>;
}

/// `namespace` corresponds to one of the elements in [`super::DataCore`]'s `namespaces`.
/// `element` corresponds to an element in the relevant sub-vector of the namespace.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Handle {
	pub(super) namespace: usize,
	pub(super) element: usize,
}

#[derive(Debug)]
pub enum Error {
	HashEmptyString,
	IdClobber,
	IdMissingPostfix,
	IdNotFound,
	NamespaceNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::HashEmptyString => {
				write!(f, "Cannot form an asset hash from an empty ID string.")
			}
			Self::IdClobber => {
				write!(f, "Attempted to overwrite an existing asset ID map key.")
			}
			Self::IdMissingPostfix => {
				write!(f, "Asset ID is malformed, and lacks a postfix.")
			}
			Self::IdNotFound => {
				write!(f, "The given asset ID did not match any existing asset.")
			}
			Self::NamespaceNotFound => {
				write!(
					f,
					"The given namespace ID did not match any existing game data object's ID."
				)
			}
		}
	}
}

bitflags::bitflags! {
	pub struct Flags: u8 {
		/// This asset was generated at run-time, rather than loaded in from the
		/// VFS. Assets without this flag are never written to save-files.
		const DYNAMIC = 1 << 0;
		/// Only assets marked `DYNAMIC` and `SAVED` are written to save-files.
		const SAVED = 1 << 1;
	}
}

pub struct Wrapper<A: Asset> {
	pub(super) inner: A,
	pub(super) _flags: Flags,
}

impl<A: Asset> Deref for Wrapper<A> {
	type Target = A;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<A: Asset> DerefMut for Wrapper<A> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

macro_rules! asset_impls {
	($({$asset_t:ty, $coll:ident, $idx:literal}),+) => {
		pub(super) const COLLECTION_COUNT: usize = 0 $(+ replace_expr!($coll 1))+;

		$(
			impl Asset for $asset_t {
				const INDEX: usize = $idx;

				fn collection(namespace: &Namespace) -> &AssetVec<Self> {
					&namespace.$coll
				}

				fn collection_mut(namespace: &mut Namespace) -> &mut AssetVec<Self> {
					&mut namespace.$coll
				}
			}
		)+
	};
}

asset_impls! {
	{ crate::game::ActorStateMachine, state_machines, 0 },
	{ crate::ecs::Blueprint, blueprints, 1 },
	{ crate::game::DamageType, damage_types, 2 },
	{ crate::level::Cluster, clusters, 3 },
	{ crate::level::Episode, episodes, 4 },
	{ crate::level::Metadata, levels, 5 },
	{ crate::game::SkillInfo, skills, 6 },
	{ crate::game::Species, species, 7 },

	{ String, language, 8 },
	{ super::Music, music, 9 },
	{ super::Sound, sounds, 10 },
	{ crate::gfx::doom::ColorMap, colormap, 11 },
	{ crate::gfx::doom::Endoom, endoom, 12 },
	{ crate::gfx::doom::Palette, palette, 13 }
}
