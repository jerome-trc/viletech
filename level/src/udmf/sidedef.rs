//! Mapping standardized sidedef field names to sidedef members and flags.

use util::SmallString;

use crate::{
	repr::{SideDef, UdmfKey, UdmfValue},
	udmf::Literal,
	Level,
};

use super::{Error, KeyValPair};

pub(super) fn read_sidedef_field(kvp: KeyValPair, level: &mut Level) -> Result<(), Error> {
	#[allow(clippy::type_complexity)]
	const KEYS_TO_CALLBACKS: &[(&str, fn(&str, &mut SideDef) -> Result<(), Error>)] = &[
			// TODO: Remaining fields for at least ZDoom.
		];

	let sidedef = level.geom.sidedefs.last_mut().unwrap();

	for (k, callback) in KEYS_TO_CALLBACKS {
		if kvp.key.eq_ignore_ascii_case(k) {
			return callback(kvp.val, sidedef);
		}
	}

	level.udmf.insert(
		UdmfKey::Sidedef {
			field: SmallString::from(kvp.key),
			index: level.geom.sidedefs.len() - 1,
		},
		match kvp.kind {
			Literal::True => UdmfValue::Bool(true),
			Literal::False => UdmfValue::Bool(false),
			Literal::Int(_) => {
				UdmfValue::Int(kvp.val.parse::<i32>().map_err(|err| Error::ParseInt {
					inner: err,
					input: kvp.val.to_string(),
				})?)
			}
			Literal::Float(_) => {
				UdmfValue::Float(kvp.val.parse::<f64>().map_err(|err| Error::ParseFloat {
					inner: err,
					input: kvp.val.to_string(),
				})?)
			}
			Literal::String(_) => UdmfValue::String(SmallString::from(kvp.val)),
		},
	);

	Ok(())
}
