use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	for path in RERUN_IF_CHANGED {
		println!("cargo:rerun-if-changed={path}");
	}

	let mut config = cbindgen::Config::default();
	config.language = cbindgen::Language::Cxx;
	config.include_version = true;
	config.namespace = Some("rs".to_string());
	config.pragma_once = true;
	config.macro_expansion.bitflags = true;
	config.structure.associated_constants_in_body = true;

	gen_header(&mut config, "", "src/lib.rs.hpp", &[])?;

	let mut ccbuild = cc::Build::new();

	ccbuild
		.includes(&["include", "src"])
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
			"src/blockmapbuilder.cpp",
			"src/classify.cpp",
			"src/events.cpp",
			"src/extract.cpp",
			"src/gl.cpp",
			"src/nodebuild.cpp",
			"src/processor.cpp",
			"src/processor_udmf.cpp",
			"src/sc_man.cpp",
			"src/utility.cpp",
			"src/wad.cpp",
			"src/zdbsp.cpp",
		]);

	if std::env::var("CARGO_FEATURE_XVERBOSE").is_ok() {
		ccbuild.define("ZDBSP_DEBUG_VERBOSE", None);
	}

	ccbuild.compile("zdbsp");

	bindgen::Builder::default()
		.header("include/zdbsp.h")
		.allowlist_item("^zdbsp_.+")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.generate()?
		.write_to_file(PathBuf::from(std::env::var("OUT_DIR")?).join("bindings.rs"))?;

	Ok(())
}

const RERUN_IF_CHANGED: &[&str] = &[
	// Rust
	"src/lib.rs",
	// Public C
	"include/zdbsp.h",
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
	"src/processor.hpp",
	"src/sc_man.cpp",
	"src/sc_man.hpp",
	"src/tarray.hpp",
	"src/templates.hpp",
	"src/utility.cpp",
	"src/wad.cpp",
	"src/wad.hpp",
	"src/xs_Float.hpp",
];

fn gen_header(
	config: &mut cbindgen::Config,
	header: &'static str,
	rel_path: &'static str,
	symbols: &'static [&'static str],
) -> Result<(), Box<dyn std::error::Error>> {
	if !header.is_empty() {
		config.header = Some(format!("//! @file\n//! @brief {header}"));
	}

	for sym in symbols {
		config.export.include.push(sym.to_string());
	}

	cbindgen::generate_with_config(std::env::var("CARGO_MANIFEST_DIR").unwrap(), config.clone())?
		.write_to_file(rel_path);

	config.export.include.clear();
	config.header = None;

	Ok(())
}
