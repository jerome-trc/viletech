use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let base = Path::new(env!("CARGO_WORKSPACE_DIR")).join("znbx");

	for rel in RERUN_IF_CHANGED {
		let path = base.join(rel);
		println!("cargo:rerun-if-changed={}", path.display());
	}

	let mut ccbuild = cc::Build::new();

	ccbuild
		.includes(&[base.join("include"), base.join("src")])
		.cpp(true)
		.flag_if_supported("-Wall")
		.flag_if_supported("-Wextra")
		.flag_if_supported("-Wpedantic")
		.flag_if_supported("-Wconversion")
		// MSVC-specific:
		.flag_if_supported("/GF") // string pooling
		.flag_if_supported("/Gy") // function-level linking
		.flag_if_supported("/GR-") // disable RTTI
		.files(&[
			base.join("src/blockmapbuilder.cpp"),
			base.join("src/classify.cpp"),
			base.join("src/events.cpp"),
			base.join("src/extract.cpp"),
			base.join("src/gl.cpp"),
			base.join("src/nodebuild.cpp"),
			base.join("src/processor.cpp"),
			base.join("src/processor_udmf.cpp"),
			base.join("src/sc_man.cpp"),
			base.join("src/utility.cpp"),
			base.join("src/wad.cpp"),
			base.join("src/znbx.cpp"),
		]);

	if std::env::var("CARGO_FEATURE_XVERBOSE").is_ok() {
		ccbuild.define("ZNBX_DEBUG_VERBOSE", None);
	}

	ccbuild.compile("znbx");

	bindgen::Builder::default()
		.header(base.join("include/znbx.h").to_string_lossy())
		.allowlist_item("^znbx_.+")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.generate()?
		.write_to_file(PathBuf::from(std::env::var("OUT_DIR")?).join("bindings.rs"))?;

	Ok(())
}

const RERUN_IF_CHANGED: &[&str] = &[
	// Rust
	"src/lib.rs",
	// Public C
	"include/znbx.h",
	// Implementation
	"src/blockmapbuilder.cpp",
	"src/blockmapbuilder.hpp",
	"src/classify.cpp",
	"src/common.hpp",
	"src/doomdata.hpp",
	"src/events.cpp",
	"src/extract.cpp",
	"src/gl.cpp",
	"src/nodebuild.cpp",
	"src/nodebuild.hpp",
	"src/processor_udmf.hpp",
	"src/processor.cpp",
	"src/processor.hpp",
	"src/sc_man.cpp",
	"src/sc_man.hpp",
	"src/tarray.hpp",
	"src/templates.hpp",
	"src/utility.cpp",
	"src/wad.cpp",
	"src/wad.hpp",
	"src/xs_Float.hpp",
	"src/znbx.cpp",
];
