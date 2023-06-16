//! Mapping standardized sectordef field names to sectordef members and flags.

use util::{id8_truncated, SmallString};

use crate::{repr::SectorDef, udmf::Value, LevelDef};

use super::{parse_i32, Error, KeyValPair};

pub(super) fn read_sectordef_field(
	kvp: KeyValPair,
	sectordef: &mut SectorDef,
	_: &LevelDef,
) -> Result<(), Error> {
	let KeyValPair { key, val } = kvp;

	match val {
		Value::Int(lit) => {
			if key.eq_ignore_ascii_case("heightfloor") {
				sectordef.height_floor = parse_i32(lit)? as f32;
			} else if key.eq_ignore_ascii_case("heightceiling") {
				sectordef.height_ceil = parse_i32(lit)? as f32;
			} else if key.eq_ignore_ascii_case("lightlevel") {
				sectordef.light_level = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("special") {
				sectordef.special = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("id") {
				sectordef.udmf_id = parse_i32(lit)?;
			} else {
				sectordef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::String(lit) => {
			if key.eq_ignore_ascii_case("texturefloor") {
				sectordef.tex_floor = Some(id8_truncated(lit));
			} else if key.eq_ignore_ascii_case("textureceiling") {
				sectordef.tex_ceil = Some(id8_truncated(lit));
			} else if key.eq_ignore_ascii_case("comment") {
				return Ok(());
			}

			sectordef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
		_ => {
			sectordef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
	}

	Ok(())
}
