use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let base = Path::new(env!("CARGO_WORKSPACE_DIR")).join("zacs");

	let mut ccbuild = cc::Build::new();

	ccbuild
		.includes(&[base.join("include"), base.join("src")])
		.cpp(true)
		.flag_if_supported("-Wall")
		.flag_if_supported("-Wextra")
		.flag_if_supported("-Wpedantic")
		.flag_if_supported("-Wconversion")
		.files(&[
			base.join("src/impl.cpp"),
			base.join("src/stb_sprintf.c"),
			base.join("src/superfasthash.cpp"),
			base.join("src/utf8.cpp"),
			base.join("src/zacs.cpp"),
			base.join("src/zstring.cpp"),
		]);

	ccbuild.compile("zacs");

	bindgen::Builder::default()
		.header(base.join("include/zacs.h").to_string_lossy())
		.allowlist_item("^zacs_.+")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.generate()?
		.write_to_file(PathBuf::from(std::env::var("OUT_DIR")?).join("bindings.rs"))?;

	Ok(())
}
