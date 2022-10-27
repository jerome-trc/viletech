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
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	fmt,
	ops::{Deref, DerefMut},
};

use fasthash::metro;
use serde::{Deserialize, Serialize};

use super::{AssetVec, Namespace};

pub trait Asset: Sized {
	const DOMAIN_STRING: &'static str;

	/// Should refer to one of the members of [`Namespace`].
	#[must_use]
	fn collection(namespace: &Namespace) -> &AssetVec<Self>;
	/// Should refer to one of the members of [`Namespace`].
	#[must_use]
	fn collection_mut(namespace: &mut Namespace) -> &mut AssetVec<Self>;
}

/// Wraps a hash, generated from an asset ID string, used as a key in
/// [`super::DataCore`]'s `asset_map`. Scripts call asset-domain-specific functions
/// and pass in strings like `"namespace:sound_id"`, so mixing in the domain's
/// string (e.g. "snd") ensures uniqueness in one hash map amongst other assets.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdHash(pub(super) u64);

impl IdHash {
	#[must_use]
	pub(super) fn from_id_pair<A: Asset>(namespace_id: &str, asset_id: &str) -> Self {
		let mut ret = metro::hash64(namespace_id);
		ret ^= metro::hash64(A::DOMAIN_STRING);
		ret ^= metro::hash64(asset_id);

		Self(ret)
	}

	pub(super) fn from_id<A: Asset>(string: &str) -> Result<Self, Error> {
		let mut split = string.split(':');

		let nsid = split.next().ok_or(Error::HashEmptyString)?;
		let aid = split.next().ok_or(Error::IdMissingPostfix)?;

		Ok(Self::from_id_pair::<A>(nsid, aid))
	}
}

/// `namespace` corresponds to one of the elements in [`super::DataCore`]'s `namespaces`.
/// `element` corresponds to an element in the relevant sub-vector of the namespace.
/// `hash` comes from the inner field of an [`IdHash`]; it's stored in save-files
/// since the ordering of elements in [`Namespace`] isn't a given.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Handle {
	#[serde(skip)]
	pub(super) namespace: usize,
	#[serde(skip)]
	pub(super) element: usize,
	pub(super) hash: u64,
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
					"The given namespace ID did not match any existing game data object's UUID."
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
	pub(super) hash: u64,
	pub(super) flags: Flags,
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

macro_rules! asset_impl {
	($asset_t:ty, $vecname:ident, $dom:literal) => {
		impl Asset for $asset_t {
			const DOMAIN_STRING: &'static str = $dom;

			fn collection(namespace: &Namespace) -> &AssetVec<Self> {
				&namespace.$vecname
			}

			fn collection_mut(namespace: &mut Namespace) -> &mut AssetVec<Self> {
				&mut namespace.$vecname
			}
		}
	};
}

asset_impl!(crate::game::ActorStateMachine, state_machines, "afsm");
asset_impl!(crate::ecs::Blueprint, blueprints, "bp");
asset_impl!(crate::game::DamageType, damage_types, "dmg_t");
asset_impl!(crate::level::Cluster, clusters, "cluster");
asset_impl!(crate::level::Episode, episodes, "episode");
asset_impl!(crate::level::Metadata, levels, "lvl");
asset_impl!(crate::game::SkillInfo, skills, "skill");
asset_impl!(crate::game::Species, species, "species");

asset_impl!(String, language, "lang");
asset_impl!(super::Music, music, "mus");
asset_impl!(super::Sound, sounds, "snd");
asset_impl!(crate::gfx::doom::ColorMap, colormap, "clrmap");
asset_impl!(crate::gfx::doom::Endoom, endoom, "endoom");
asset_impl!(crate::gfx::doom::Palette, palette, "playpal");
