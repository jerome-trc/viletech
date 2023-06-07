//! Mapping standardized sidedef field names to sidedef members and flags.

use util::id8_truncated;

use crate::{repr::SideDef, udmf::Value, Level};

use super::{parse_i32, parse_usize, Error, KeyValPair};

pub(super) fn read_sidedef_field(
	kvp: KeyValPair,
	sidedef: &mut SideDef,
	level: &mut Level,
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
				level.udmf.insert(
					kvp.to_sidedef_mapkey(level.geom.sidedefs.len()),
					kvp.to_map_value(),
				);
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
				level.udmf.insert(
					kvp.to_sidedef_mapkey(level.geom.sidedefs.len()),
					kvp.to_map_value(),
				);
			}
		}
		_ => {
			level.udmf.insert(
				kvp.to_sidedef_mapkey(level.geom.sidedefs.len()),
				kvp.to_map_value(),
			);
		}
	}

	Ok(())
}
