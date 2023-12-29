fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut config = cbindgen::Config::default();
	config.language = cbindgen::Language::Cxx;
	config.include_version = true;
	config.namespace = Some("rs".to_string());
	config.pragma_once = true;
	config.macro_expansion.bitflags = true;
	config.structure.associated_constants_in_body = true;

	gen_header(&mut config, "", "src/lib.rs.hpp", &[])?;
}

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
