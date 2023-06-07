//! Mapping standardized thingdef field names to thingdef members and flags.

use util::SmallString;

use crate::{
	repr::{LevelFormat, Thing, ThingFlags, UdmfNamespace},
	udmf::Value,
	Level,
};

use super::{parse_f64, parse_i32, parse_u16, parse_u32, Error, KeyValPair};

pub(super) fn read_thingdef_field(
	kvp: KeyValPair,
	thing: &mut Thing,
	level: &Level,
) -> Result<(), Error> {
	let KeyValPair { key, val } = kvp;

	match val {
		Value::True | Value::False => {
			if key.eq_ignore_ascii_case("skill1") {
				thing
					.flags
					.set(ThingFlags::SKILL_1, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill2") {
				thing
					.flags
					.set(ThingFlags::SKILL_2, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill3") {
				thing
					.flags
					.set(ThingFlags::SKILL_3, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill4") {
				thing
					.flags
					.set(ThingFlags::SKILL_4, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("skill5") {
				thing
					.flags
					.set(ThingFlags::SKILL_5, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("ambush") {
				thing
					.flags
					.set(ThingFlags::AMBUSH, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("single") {
				thing
					.flags
					.set(ThingFlags::SINGLEPLAY, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dm") {
				thing
					.flags
					.set(ThingFlags::DEATHMATCH, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("coop") {
				thing
					.flags
					.set(ThingFlags::COOP, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("friend") {
				thing
					.flags
					.set(ThingFlags::FRIEND, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("dormant") {
				thing
					.flags
					.set(ThingFlags::DORMANT, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class1") {
				thing
					.flags
					.set(ThingFlags::CLASS_1, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class2") {
				thing
					.flags
					.set(ThingFlags::CLASS_2, matches!(val, Value::True));
			} else if key.eq_ignore_ascii_case("class3") {
				thing
					.flags
					.set(ThingFlags::CLASS_3, matches!(val, Value::True));
			} else if matches!(level.format, LevelFormat::Udmf(UdmfNamespace::Strife)) {
				unimplemented!("The `Strife` UDMF namespace is not yet supported.");
			} else {
				thing
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Int(lit) => {
			if key.eq_ignore_ascii_case("id") {
				thing.tid = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("angle") {
				thing.angle = parse_u32(lit)?;
			} else if key.eq_ignore_ascii_case("type") {
				thing.ed_num = parse_u16(lit)?;
			} else if key.eq_ignore_ascii_case("special") {
				thing.special = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg0") {
				thing.args[0] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg1") {
				thing.args[1] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg2") {
				thing.args[2] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg3") {
				thing.args[3] = parse_i32(lit)?;
			} else if key.eq_ignore_ascii_case("arg4") {
				thing.args[4] = parse_i32(lit)?;
			} else {
				thing
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::Float(lit) => {
			let float = parse_f64(lit)?;

			// Recall that Y is up in VileTech.
			if key.eq_ignore_ascii_case("x") {
				thing.pos.x = float as f32;
			} else if key.eq_ignore_ascii_case("y") {
				thing.pos.z = float as f32;
			} else if key.eq_ignore_ascii_case("z") {
				thing.pos.y = float as f32;
			} else {
				thing
					.udmf
					.insert(SmallString::from(kvp.key), kvp.to_map_value());
			}
		}
		Value::String(_) => {
			if key.eq_ignore_ascii_case("comment") {
				return Ok(());
			}

			thing
				.udmf
				.insert(SmallString::from(kvp.key), kvp.to_map_value());
		}
	}

	Ok(())
}
