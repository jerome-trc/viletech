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

use clap::Parser;
use impure::{depends::*, data::game::DataCore};
use log::info;
use std::error::Error;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
	// ???
}

fn main() -> Result<(), Box<dyn Error>> {
	let _args = Args::parse();

	match impure::log_init(None) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {}", err);
			return Err(err);
		}
	}

	info!("{}", impure::short_version_string());
	info!("Impure dedicated server version {}.", env!("CARGO_PKG_VERSION"));
	info!("{}", impure::utils::env::os_info()?);

	let _data = DataCore::default();

	Ok(())
}
