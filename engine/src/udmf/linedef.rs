//! Mapping standardized linedef field names to linedef members and flags.

use util::SmallString;

use crate::{
	data::dobj::{Level, LineDef, UdmfKey, UdmfValue},
	udmf::Literal,
};

use super::{Error, KeyValPair};

pub(super) fn read_linedef_field(kvp: KeyValPair, level: &mut Level) -> Result<(), Error> {
	#[allow(clippy::type_complexity)]
	const KEYS_TO_CALLBACKS: &[(&str, fn(&str, &mut LineDef) -> Result<(), Error>)] = &[
			// TODO: Remaining fields for at least ZDoom.
		];

	let linedef = level.linedefs.last_mut().unwrap();

	for (k, callback) in KEYS_TO_CALLBACKS {
		if kvp.key.eq_ignore_ascii_case(k) {
			return callback(kvp.val, linedef);
		}
	}

	level.udmf.insert(
		UdmfKey::Linedef {
			field: SmallString::from(kvp.key),
			index: level.linedefs.len() - 1,
		},
		match kvp.kind {
			Literal::True => UdmfValue::Bool(true),
			Literal::False => UdmfValue::Bool(false),
			Literal::Int => {
				UdmfValue::Int(kvp.val.parse::<i32>().map_err(|err| Error::ParseInt {
					inner: err,
					input: kvp.val.to_string(),
				})?)
			}
			Literal::Float => {
				UdmfValue::Float(kvp.val.parse::<f64>().map_err(|err| Error::ParseFloat {
					inner: err,
					input: kvp.val.to_string(),
				})?)
			}
			Literal::String => UdmfValue::String(SmallString::from(kvp.val)),
		},
	);

	Ok(())
}
