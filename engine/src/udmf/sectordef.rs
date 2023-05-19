//! Mapping standardized sectordef field names to sectordef members and flags.

use crate::{
	data::dobj::{Level, Sector, UdmfKey, UdmfValue},
	udmf::Literal,
	SmallString,
};

use super::{Error, KeyValPair};

pub(super) fn read_sectordef_field(kvp: KeyValPair, level: &mut Level) -> Result<(), Error> {
	#[allow(clippy::type_complexity)]
	const KEYS_TO_CALLBACKS: &[(&str, fn(&str, &mut Sector) -> Result<(), Error>)] = &[
			// TODO: Remaining fields for at least ZDoom.
		];

	let sector = level.sectors.last_mut().unwrap();

	for (k, callback) in KEYS_TO_CALLBACKS {
		if kvp.key.eq_ignore_ascii_case(k) {
			return callback(kvp.val, sector);
		}
	}

	level.udmf.insert(
		UdmfKey::Sector {
			field: SmallString::from(kvp.key),
			index: level.sectors.len() - 1,
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
