use std::{error::Error, process::Command};

/// Injects the current Git hash and date and time of compilation
/// into the environment before building.
fn main() -> Result<(), Box<dyn Error>> {
	let hash = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
		Ok(h) => h,
		Err(err) => {
			eprintln!("Failed to execute `git rev-parse HEAD`: {}", err);
			return Err(Box::new(err));
		}
	};

	let hash_str = match String::from_utf8(hash.stdout) {
		Ok(s) => s,
		Err(err) => {
			eprintln!(
				"Failed to convert output of `git rev-parse HEAD` to UTF-8: {}",
				err
			);
			return Err(Box::new(err));
		}
	};

	println!("cargo:rustc-env=GIT_HASH={}", hash_str);
	println!(
		"cargo:rustc-env=COMPILE_DATETIME={} UTC",
		chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
	);

	Ok(())
}
