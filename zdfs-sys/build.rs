use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	if cfg!(any(target_os = "macos", target_os = "freebsd")) {
		println!("cargo:rustc-link-lib=static=c++");
	} else {
		println!("cargo:rustc-link-lib=static=stdc++");
		println!("cargo:rustc-link-lib=static=gcc");
	}

	let cmake_root = Path::new(env!("CARGO_WORKSPACE_DIR")).join("zdfs");

	let cmake_out = cmake::Config::new(&cmake_root)
		.define("BUILD_SHARED_LIBS", "OFF")
		.build();

	let incl_path = cmake_root.join("include");
	let lib_path = cmake_out.join("build");

	println!("cargo:include={}", incl_path.display());
	println!("cargo:rustc-link-search=native={}", lib_path.display());

	println!("cargo:rustc-link-lib=static=zdfs");

	std::fs::copy(
		cmake_out.join("build/compile_commands.json"),
		cmake_root.join("compile_commands.json"),
	)?;

	bindgen::Builder::default()
		.header(incl_path.join("zdfs.h").to_str().unwrap_or_else(|| {
			panic!("failed to convert include header path to a string");
		}))
		.blocklist_file(".*stdlib.h")
		.blocklist_file(".*stdint.h")
		.blocklist_file(".*stdarg.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.generate()
		.map_err(|err| {
			eprintln!("Binding generation failed.");
			err
		})?
		.write_to_file(
			Path::new(&std::env::var("OUT_DIR").map_err(|err| {
				eprintln!("Failed to retrieve path to `OUT_DIR`.");
				err
			})?)
			.join("bindings.rs"),
		)
		.map_err(|err| {
			eprintln!("Failed to join path `$OUT_DIR/bindings.rs`.");
			err
		})?;

	Ok(())
}
