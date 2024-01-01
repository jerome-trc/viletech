use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	if cfg!(any(target_os = "macos", target_os = "freebsd")) {
		println!("cargo:rustc-link-lib=static=c++");
	} else {
		println!("cargo:rustc-link-lib=static=stdc++");
		println!("cargo:rustc-link-lib=static=gcc");
	}

	let cmake_root = Path::new(env!("CARGO_WORKSPACE_DIR")).join("acsvm");

	let cmake_out = cmake::Config::new(&cmake_root)
		.define("CMAKE_EXPORT_COMPILE_COMMANDS", "1")
		.define("ACSVM_SHARED", "OFF")
		.build();

	let incl_path = cmake_root.join("CAPI");
	let lib_path = cmake_out.join("lib");

	println!("cargo:include={}", incl_path.display());
	println!("cargo:rustc-link-search=native={}", lib_path.display());

	println!("cargo:rustc-link-lib=static=acsvm");

	std::fs::copy(
		cmake_out.join("build/compile_commands.json"),
		cmake_root.join("compile_commands.json"),
	)?;

	let mut builder = bindgen::Builder::default();

	for incl in HEADERS {
		builder = builder.header(incl_path.join(incl).to_string_lossy());
	}

	builder
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

const HEADERS: &[&str] = &[
	"Array.h",
	"BinaryIO.h",
	"Code.h",
	"Environment.h",
	"Floats.h",
	"Module.h",
	"PrintBuf.h",
	"Scope.h",
	"Script.h",
	"String.h",
	"Thread.h",
	"Types.h",
];
