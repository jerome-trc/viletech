//! Mapping standardized sidedef field names to sidedef members and flags.

use util::{id8_truncated, SmallString};

use crate::{repr::SideDef, udmf::Value, LevelDef};

use super::{parse_i32, parse_usize, Error, KeyValPair};

pub(super) fn read_sidedef_field(
	kvp: KeyValPair,
	sidedef: &mut SideDef,
	_: &LevelDef,
) -> Result<(), Error> {
	let KeyValPair { key, val } = kvp;

	match val {
		Value::String(lit) => {
			if key.eq_ignore_ascii_case("texturetop") {
				sidedef.tex_top = Some(id8_truncated(lit));
			} else if key.eq_ignore_ascii_case("texturebottom") {
				sidedef.tex_bottom = Some(id8_truncated(lit));
			} else if key.eq_ignore_ascii_case("texturemiddle") {
				sidedef.tex_mid = Some(id8_truncated(lit));
			} else if key.eq_ignore_ascii_case("comment") {
				return Ok(());
			} else {
				sidedef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Int(lit) => {
			if key.eq_ignore_ascii_case("sector") {
				sidedef.sector = parse_usize(lit)?;
			} else if key.eq_ignore_ascii_case("offsetx") {
				sidedef.offset.x = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("offsety") {
				sidedef.offset.y = parse_i32(lit)?;
			} else {
				sidedef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		_ => {
			sidedef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
	}

	Ok(())
}
