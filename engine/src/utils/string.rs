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

use lazy_static::lazy_static;
use regex::Regex;

/// Extracts a version string from what will almost always be a file stem,
/// using a search pattern based off the most common versioning conventions used
/// in ZDoom modding. If the returned option is `None`, the given string is unmodified.
pub fn version_from_string(string: &mut String) -> Option<String> {
	lazy_static! {
		static ref RGX_VERSION: Regex = Regex::new(
			r"(?x)
			(?:[VR]|[\s\-_][VvRr]|[\s\-_\.])\d{1,}
			(?:[\._\-]\d{1,})*
			(?:[\._\-]\d{1,})*
			[A-Za-z]*[\._\-]*
			[A-Za-z0-9]*
			$"
		)
		.expect("Failed to evaluate `utils::version_from_string::RGX_VERSION`.");
	}

	match RGX_VERSION.find(string) {
		Some(m) => {
			const TO_TRIM: [char; 3] = [' ', '_', '-'];
			let ret = m.as_str().trim_matches(&TO_TRIM[..]).to_string();
			string.replace_range(m.range(), "");
			Some(ret)
		}
		None => None,
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn version_from_string() {
		let mut input = [
			"DOOM".to_string(),
			"DOOM2".to_string(),
			"weighmedown_1.2.0".to_string(),
			"none an island V1.0".to_string(),
			"bitter_arcsV0.9.3".to_string(),
			"stuck-in-the-system_v3.1".to_string(),
			"555-5555 v5a".to_string(),
			"yesterdays_pain_6-19-2022".to_string(),
			"i-am_a dagger_1.3".to_string(),
			"BROKEN_MANTRA_R3.1c".to_string(),
			"There Is Still Time 0.3tf1".to_string(),
			"setmefree-4.7.1c".to_string(),
			"Outoftheframe_1_7_0b".to_string(),
			"a c i d r a i n_716".to_string()
		];

		let expected = [
			("DOOM", ""),
			("DOOM2", ""),
			("weighmedown", "1.2.0"),
			("none an island", "V1.0"),
			("bitter_arcs", "V0.9.3"),
			("stuck-in-the-system", "v3.1"),
			("555-5555", "v5a"),
			("yesterdays_pain", "6-19-2022"),
			("i-am_a dagger", "1.3"),
			("BROKEN_MANTRA", "R3.1c"),
			("There Is Still Time", "0.3tf1"),
			("setmefree", "4.7.1c"),
			("Outoftheframe", "1_7_0b"),
			("a c i d r a i n", "716")
		];

		for (i, string) in input.iter_mut().enumerate() {
			let res = super::version_from_string(string);

			if expected[i].1.is_empty() {
				assert!(res.is_none(), "[{i}] - Expected nothing; returned: {}", res.unwrap());
			} else {
				assert!(
					string == expected[i].0,
					"[{i}] Expected modified string: {}
					Actual output: {}", expected[i].0, string
				);

				let vers = res.unwrap();

				assert!(
					vers == expected[i].1,
					"[{i}] Expected return value: {}
					Actual return value: {}", expected[i].1, vers 
				);
			}
		}
	}
}
