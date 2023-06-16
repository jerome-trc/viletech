//! Mapping standardized thingdef field names to thingdef members and flags.

use util::SmallString;

use crate::{
	repr::{LevelFormat, ThingDef, ThingFlags, UdmfNamespace},
	udmf::Value,
	LevelDef,
};

use super::{parse_f64, parse_i32, parse_u16, parse_u32, Error, KeyValPair};

pub(super) fn read_thingdef_field(
	kvp: KeyValPair,
	thingdef: &mut ThingDef,
	level: &LevelDef,
) -> Result<(), Error> {
	let KeyValPair { key, val } = kvp;

	match val {
		Value::True | Value::False => {
			if key.eq_ignore_ascii_case("skill1") {
				thingdef
					.flags
					.set(ThingFlags::SKILL_1, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill2") {
				thingdef
					.flags
					.set(ThingFlags::SKILL_2, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill3") {
				thingdef
					.flags
					.set(ThingFlags::SKILL_3, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill4") {
				thingdef
					.flags
					.set(ThingFlags::SKILL_4, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill5") {
				thingdef
					.flags
					.set(ThingFlags::SKILL_5, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("ambush") {
				thingdef
					.flags
					.set(ThingFlags::AMBUSH, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("single") {
				thingdef
					.flags
					.set(ThingFlags::SINGLEPLAY, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dm") {
				thingdef
					.flags
					.set(ThingFlags::DEATHMATCH, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("coop") {
				thingdef
					.flags
					.set(ThingFlags::COOP, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("friend") {
				thingdef
					.flags
					.set(ThingFlags::FRIEND, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dormant") {
				thingdef
					.flags
					.set(ThingFlags::DORMANT, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class1") {
				thingdef
					.flags
					.set(ThingFlags::CLASS_1, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class2") {
				thingdef
					.flags
					.set(ThingFlags::CLASS_2, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class3") {
				thingdef
					.flags
					.set(ThingFlags::CLASS_3, matches!(val, Value::True));
			} else if matches!(level.format, LevelFormat::Udmf(UdmfNamespace::Strife)) {
				unimplemented!("the `Strife` UDMF namespace is not yet supported");
			} else {
				thingdef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Int(lit) => {
			if key.eq_ignore_ascii_case("id") {
				thingdef.tid = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("angle") {
				thingdef.angle = parse_u32(lit)?;
			} else if key.eq_ignore_ascii_case("type") {
				thingdef.ed_num = parse_u16(lit)?;
			} else if key.eq_ignore_ascii_case("special") {
				thingdef.special = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg0") {
				thingdef.args[0] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg1") {
				thingdef.args[1] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg2") {
				thingdef.args[2] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg3") {
				thingdef.args[3] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg4") {
				thingdef.args[4] = parse_i32(lit)?;
			} else {
				thingdef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Float(lit) => {
			let float = parse_f64(lit)?;

			// Recall that Y is up in VileTech.
			if key.eq_ignore_ascii_case("x") {
				thingdef.pos.x = float as f32;
			} else if key.eq_ignore_ascii_case("y") {
				thingdef.pos.z = float as f32;
			} else if key.eq_ignore_ascii_case("z") {
				thingdef.pos.y = float as f32;
			} else {
				thingdef
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::String(_) => {
			if key.eq_ignore_ascii_case("comment") {
				return Ok(());
			}

			thingdef
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
	}

	Ok(())
}
