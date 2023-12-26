use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	for path in RERUN_IF_CHANGED.iter().copied() {
		println!("cargo:rerun-if-changed={path}");
	}

	let mut build = cxx_build::bridge("src/lib.rs");

	build
		.include("include")
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
			"src/utility.cpp",
			"src/wad.cpp",
		]);

	build.compile("zdbsp");

	let gen_header =
		Path::new(&std::env::var("OUT_DIR")?).join("cxxbridge/include/zdbsp/src/lib.rs.h");

	std::fs::copy(&gen_header, "include/lib.rs.hpp")?;

	Ok(())
}

const RERUN_IF_CHANGED: &[&str] = &[
	"src/lib.rs",
	"include/zdbsp.hpp",
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
	"src/resource.hpp",
	"src/sc_man.cpp",
	"src/sc_man.hpp",
	"src/tarray.hpp",
	"src/templates.hpp",
	"src/utility.cpp",
	"src/wad.cpp",
	"src/wad.hpp",
	"src/workdata.hpp",
	"src/xs_Float.hpp",
];
