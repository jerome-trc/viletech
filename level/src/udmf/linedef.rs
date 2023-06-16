//! Mapping standardized linedef field names to linedef members and flags.

use util::SmallString;

use crate::{
	repr::{LineDef, LineFlags},
	udmf::Value,
	LevelDef,
};

use super::{parse_i32, parse_usize, Error, KeyValPair};

pub(super) fn read_linedef_field(
	kvp: KeyValPair,
	linedef: &mut LineDef,
	_: &LevelDef,
) -> Result<(), Error> {
	let KeyValPair { key, val } = kvp;

	match val {
		Value::False | Value::True => {
			if key.eq_ignore_ascii_case("blocking") {
				linedef
					.flags
					.set(LineFlags::IMPASSIBLE, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("blockmonsters") {
				linedef
					.flags
					.set(LineFlags::BLOCK_MONS, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("twosided") {
				linedef
					.flags
					.set(LineFlags::TWO_SIDED, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dontpegtop") {
				linedef
					.flags
					.set(LineFlags::UPPER_UNPEGGED, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dontpegbottom") {
				linedef
					.flags
					.set(LineFlags::LOWER_UNPEGGED, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("secret") {
				linedef
					.flags
					.set(LineFlags::SECRET, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("blocksound") {
				linedef
					.flags
					.set(LineFlags::BLOCK_SOUND, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dontdraw") {
				linedef
					.flags
					.set(LineFlags::UNMAPPED, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("mapped") {
				linedef
					.flags
					.set(LineFlags::PRE_MAPPED, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("passuse") {
				linedef
					.flags
					.set(LineFlags::PASS_USE, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("translucent") {
				linedef
					.flags
					.set(LineFlags::TRANSLUCENT, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("jumpover") {
				linedef
					.flags
					.set(LineFlags::JUMPOVER, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("blockfloaters") {
				linedef
					.flags
					.set(LineFlags::BLOCK_FLOATERS, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("playercross") {
				linedef
					.flags
					.set(LineFlags::ALLOW_PLAYER_CROSS, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("playeruse") {
				linedef
					.flags
					.set(LineFlags::ALLOW_PLAYER_USE, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("monsterpush") {
				linedef
					.flags
					.set(LineFlags::ALLOW_MONS_PUSH, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("missilecross") {
				linedef
					.flags
					.set(LineFlags::ALLOW_PROJ_CROSS, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("repeatspecial") {
				linedef
					.flags
					.set(LineFlags::REPEAT_SPECIAL, matches!(val, Value::True));
			} else {
				linedef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Int(lit) => {
			if key.eq_ignore_ascii_case("id") {
				linedef.udmf_id = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("v1") {
				linedef.vert_start = parse_usize(lit)?;
			} else if key.eq_ignore_ascii_case("v2") {
				linedef.vert_end = parse_usize(lit)?;
			} else if key.eq_ignore_ascii_case("special") {
				linedef.special = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg0") {
				linedef.args[0] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg1") {
				linedef.args[1] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg2") {
				linedef.args[2] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg3") {
				linedef.args[3] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg4") {
				linedef.args[4] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("sidefront") {
				linedef.side_right = parse_usize(lit)?;
			} else if key.eq_ignore_ascii_case("sideback") {
				linedef.side_left = Some(parse_usize(lit)?);
			} else {
				linedef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::String(_) => {
			if key.eq_ignore_ascii_case("comment") {
				return Ok(());
			}

			linedef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
		_ => {
			linedef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
	}

	Ok(())
}
