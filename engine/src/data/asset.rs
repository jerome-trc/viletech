//! A trait providing generic functionality for [`super::game::DataCore`].

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

use crate::{
	ecs::Blueprint,
	game::DamageType,
	game::{SkillInfo, Species},
	gfx::doom::{ColorMap, Endoom, Palette},
	level::Episode,
	LevelCluster, LevelMetadata, Namespace,
};

use super::game::{Music, Sound};

pub trait Asset {
	/// For "singleton" assets like palettes, this should be left empty.
	const DOMAIN_STRING: &'static str;

	/// Returns the index of the asset in the vector that newly holds it.
	/// If the asset doesn't go into a vector, return 0.
	fn add_impl(namespace: &mut Namespace, asset: Self) -> usize;
	fn get_impl<'lua>(namespace: &'lua Namespace, index: usize) -> Option<&'lua Self>;
}

macro_rules! asset_vec {
	($asset_t:ty, $vecname:ident, $dom:literal) => {
		impl Asset for $asset_t {
			const DOMAIN_STRING: &'static str = $dom;

			fn add_impl(namespace: &mut Namespace, asset: Self) -> usize {
				namespace.$vecname.push(asset);
				namespace.$vecname.len() - 1
			}

			fn get_impl<'lua>(namespace: &'lua Namespace, index: usize) -> Option<&'lua Self> {
				namespace.$vecname.get(index)
			}
		}
	};
}

macro_rules! asset_opt {
	($asset_t:ty, $optname:ident) => {
		impl Asset for $asset_t {
			const DOMAIN_STRING: &'static str = "";

			fn add_impl(namespace: &mut Namespace, asset: Self) -> usize {
				namespace.$optname = Some(asset);
				0
			}

			fn get_impl<'lua>(namespace: &'lua Namespace, _index: usize) -> Option<&'lua Self> {
				namespace.$optname.as_ref()
			}
		}
	};
}

asset_vec!(Blueprint, blueprints, "bp");
asset_vec!(DamageType, damage_types, "dmg_t");
asset_vec!(LevelCluster, clusters, "cluster");
asset_vec!(Episode, episodes, "episode");
asset_vec!(LevelMetadata, levels, "lvl");
asset_vec!(SkillInfo, skills, "skill");
asset_vec!(Species, species, "species");
asset_vec!(String, language, "lang");
asset_vec!(Music, music, "mus");
asset_vec!(Sound, sounds, "snd");

asset_opt!(ColorMap, colormap);
asset_opt!(Endoom, endoom);
asset_opt!(Palette, palette);
