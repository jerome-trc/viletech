//! # VileTech Engine
//!
//! VileTech is a pet project, meant as an experiment in creating a modern,
//! feature-oriented Doom source port in Rust which can successfully interpret
//! user-generated content for GZDoom and the Eternity Engine with zero end-user
//! overhead and minimal runtime overhead.

#[allow(dead_code)]
pub mod acs;
pub mod audio;
pub mod console;
pub mod data;
#[allow(dead_code)]
pub mod frontend;
#[allow(dead_code)]
pub mod game;
pub mod gfx;
pub mod input;
pub mod lith;
pub mod math;
pub mod player;
#[allow(dead_code)]
pub mod rng;
#[allow(dead_code)]
pub mod sim;
pub mod simd;
pub mod sparse;
pub mod terminal;
#[allow(dead_code)]
pub mod user;
pub mod utils;
pub mod wad;

// Type aliases

/// Disambiguates between real FS paths and virtual FS paths.
pub type VPath = std::path::Path;
/// Disambiguates between real FS paths and virtual FS paths.
pub type VPathBuf = std::path::PathBuf;
/// WAD entries have a name length limit of 8 ASCII characters, and this limitation
/// has persisted through Doom's descendant source ports (for whatever reason).
/// For compatibility purposes, VileTech sometimes needs to pretend that there's
/// no game data namespacing and look up the last loaded thing with a certain name.
pub type ShortId = arrayvec::ArrayString<8>;

/// See <https://zdoom.org/wiki/Editor_number>. Used when populating levels.
pub type EditorNum = u16;
/// See <https://zdoom.org/wiki/Spawn_number>. Used by ACS.
pub type SpawnNum = u16;

// Symbols that don't belong in any other module ///////////////////////////////

/// State and functions for a two-panel egui window that sticks to the top of the
/// screen like GZDoom's console. `S` should be a simple untagged enum that
/// informs the user what they should draw in each panel.
pub struct DeveloperGui<S: PartialEq + Copy> {
	pub open: bool,
	pub left: S,
	pub right: S,
}

impl<S: PartialEq + Copy> DeveloperGui<S> {
	/// Returns an egui window that:
	/// - Is 80% opaque
	/// - Stretches to fill the screen's width
	/// - Is immovably anchored to the screen's top
	/// - Can be resized, but only vertically
	pub fn window(ctx: &egui::Context) -> egui::containers::Window {
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		egui::Window::new("Developer Tools")
			.id(egui::Id::new("vile_devgui"))
			.anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
			.fixed_pos([0.0, 0.0])
			.collapsible(false)
			.resizable(true)
			.min_width(screen_rect.width())
			.min_height(screen_rect.height() * 0.1)
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	pub fn panel_left(&self, ctx: &egui::Context) -> egui::SidePanel {
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		egui::SidePanel::left("vile_devgui_left")
			.default_width(screen_rect.width() * 0.5)
			.resizable(true)
			.width_range((screen_rect.width() * 0.1)..=(screen_rect.width() * 0.9))
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	/// Ensure this is only called after [`panel_left`](DeveloperGui::panel_left).
	pub fn panel_right(&self, ctx: &egui::Context) -> egui::CentralPanel {
		egui::CentralPanel::default()
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
	}

	/// Call after opening the [developer GUI window](DeveloperGui::window).
	/// Draws two dropdowns in its menu bar that allow changing which menu is
	/// being drawn in each pane. A menu can't replace itself, but the left and
	/// right side can be swapped.
	pub fn selectors(&mut self, ui: &mut egui::Ui, choices: &[(S, &str)]) {
		egui::menu::bar(ui, |ui| {
			ui.menu_button("Left", |ui| {
				for (choice, label) in choices {
					let btn = egui::Button::new(*label);
					let resp = ui.add_enabled(self.left != *choice, btn);

					if resp.clicked() {
						ui.close_menu();

						if self.right == *choice {
							std::mem::swap(&mut self.left, &mut self.right);
						} else {
							self.left = *choice;
						}
					}
				}
			});

			ui.menu_button("Right", |ui| {
				for (choice, label) in choices {
					let btn = egui::Button::new(*label);
					let resp = ui.add_enabled(self.right != *choice, btn);

					if resp.clicked() {
						ui.close_menu();

						if self.left == *choice {
							std::mem::swap(&mut self.left, &mut self.right);
						} else {
							self.right = *choice;
						}
					}
				}
			});
		});
	}
}

/// Prepares the rayon global thread pool. See [`rayon::ThreadPoolBuilder`].
/// If `num_threads` is `None` then rayon chooses it automatically.
/// This also ensures that these threads have clear names for debugging purposes.
pub fn thread_pool_init(num_threads: Option<usize>) {
	rayon::ThreadPoolBuilder::new()
		.thread_name(|index| format!("vile-global-{index}"))
		.num_threads(num_threads.unwrap_or(0))
		.build_global()
		.expect("Failed to build rayon's global thread pool.")
}

/// After initializing [`log`], call this to print:
/// - The engine's semantic version.
/// - The application's semantic version.
/// - Information about the operating system.
/// - Information about the CPU's relevant properties.
pub fn log_init_diag(app_version_string: &str) -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
	fn log_cpu_info() -> Result<(), Box<dyn std::error::Error>> {
		unimplemented!("CPU diagnostics logging only available on x86(_64).");
	}

	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	fn log_cpu_info() -> Result<(), Box<dyn std::error::Error>> {
		let cpuid = raw_cpuid::CpuId::new();
		let mut output = String::with_capacity(512);

		if let Some(vendor) = cpuid.get_vendor_info() {
			output.push_str(&format!("\t- Vendor ID: \"{}\"\r\n", vendor.as_str()))
		} else {
			output.push_str("\t- Vendor ID: <unknown>\r\n");
		};

		if let Some(pbs) = cpuid.get_processor_brand_string() {
			output.push_str(&format!("\t- Name: \"{}\"\r\n", pbs.as_str()));
		} else {
			output.push_str("\t- Name: <unknown>\r\n");
		}

		if let Some(feats) = cpuid.get_feature_info() {
			output.push_str(&format!(
				"\t- Family ID: {} ({} base, {} extended)
	\t- Model ID: {} ({} base, {} extended)
	\t- Stepping ID: {}\r\n",
				feats.family_id(),
				feats.base_family_id(),
				feats.extended_family_id(),
				feats.model_id(),
				feats.base_model_id(),
				feats.extended_model_id(),
				feats.stepping_id()
			));

			output.push_str("\t- Features:");

			if feats.has_avx() {
				output.push_str(" AVX");
			}

			if feats.has_f16c() {
				output.push_str(" F16C");
			}

			if feats.has_fma() {
				output.push_str(" FMA");
			}

			if feats.has_sse() {
				output.push_str(" SSE");
			}

			if feats.has_sse2() {
				output.push_str(" SSE2");
			}

			if feats.has_sse3() {
				output.push_str(" SSE3");
			}

			if feats.has_sse41() {
				output.push_str(" SSE4.1");
			}

			if feats.has_sse42() {
				output.push_str(" SSE4.2");
			}

			if output.ends_with("Features:") {
				output.push_str(" <none>\r\n");
			} else {
				output.push_str("\r\n");
			}
		} else {
			output.push_str("\t- Feature/family information unavailable\r\n");
		}

		if let Some(extfeats) = cpuid.get_extended_feature_info() {
			output.push_str("\t- Extended features:");

			if extfeats.has_avx2() {
				output.push_str(" AVX2");
			}

			if output.ends_with("Extended features:") {
				output.push_str(" <none>\r\n");
			} else {
				output.push_str("\r\n");
			}
		} else {
			output.push_str("\t- Extended feature information unavailable\r\n");
		}

		let avail_par = std::thread::available_parallelism()?;
		output.push_str(&format!("\t- Available parallelism: {avail_par}"));

		log::info!("CPU diagnostics: \r\n{}", output);

		Ok(())
	}

	log::info!("{}", short_version_string());
	log::info!("{}", app_version_string);
	log::info!("{}", utils::env::os_info()?);
	log_cpu_info()?;

	Ok(())
}

/// Returns a message telling the user the engine's `CARGO_PKG_VERSION`.
#[must_use]
pub fn short_version_string() -> String {
	format!("VileTech Engine version: {}", env!("CARGO_PKG_VERSION"))
}

/// Returns a message telling the user the engine's `CARGO_PKG_VERSION`, the version
/// of the application running on the engine, the SHA hash of the Git commit from
/// which the engine was built, and the date and time of engine compilation.
#[must_use]
pub fn full_version_string(app_version_string: &str) -> String {
	format!(
		"VileTech Engine {}\r\n\t{app_version_string}\
		\r\n\tGit commit: {}\r\n\tCompiled on: {}",
		env!("CARGO_PKG_VERSION"),
		env!("GIT_HASH"),
		env!("COMPILE_DATETIME")
	)
}

/// Prepares the fern logging backend.
pub fn log_init(
	sender: Option<crossbeam::channel::Sender<console::Message>>,
) -> Result<(), Box<dyn std::error::Error>> {
	use console::Writer;
	use std::{
		fs, io,
		path::{Path, PathBuf},
	};

	let exe_dir = utils::path::exe_dir();

	let colors = fern::colors::ColoredLevelConfig::new()
		.info(fern::colors::Color::Green)
		.warn(fern::colors::Color::Yellow)
		.error(fern::colors::Color::Red)
		.debug(fern::colors::Color::Cyan)
		.trace(fern::colors::Color::Magenta);

	let fpath: PathBuf = [&exe_dir, Path::new("viletech.log")].iter().collect();

	if fpath.exists() {
		let oldpath: PathBuf = [&exe_dir, Path::new("viletech.log.old")].iter().collect();

		if let Err(err) = fs::rename(&fpath, oldpath) {
			eprintln!("Failed to rotate previous log file: {err}");
		}
	}

	let file_cfg = fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
				record.target(),
				record.level(),
				message
			))
		})
		.chain(
			fs::OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(fpath)?,
		);

	// Stdout logging has console colouring and less date-time elaboration
	let stdout_cfg = fern::Dispatch::new()
		.format(move |out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%H:%M:%S]"),
				record.target(),
				colors.color(record.level()),
				message
			))
		})
		.chain(io::stdout());

	let dispatch = fern::Dispatch::new()
		.level(log::LevelFilter::Trace)
		.level_for("naga", log::LevelFilter::Warn)
		.level_for("wgpu_hal", log::LevelFilter::Error)
		.level_for("wgpu_core", log::LevelFilter::Error)
		.level_for("symphonia_core", log::LevelFilter::Warn)
		.level_for("symphonia_format_ogg", log::LevelFilter::Warn)
		.level_for("symphonia_codec_vorbis", log::LevelFilter::Warn)
		.level_for("symphonia_bundle_mp3", log::LevelFilter::Warn)
		.chain(file_cfg)
		.chain(stdout_cfg);

	if let Some(s) = sender {
		let console_cfg = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!("[{}] {}", record.level(), message))
			})
			.chain(Box::new(Writer::new(s)) as Box<dyn io::Write + Send>);

		dispatch.chain(console_cfg).apply()
	} else {
		dispatch.apply()
	}?;

	Ok(())
}

pub const BASEDATA_ID: &str = env!("BASEDATA_ID");
pub const BASEDATA_FILENAME: &str = env!("BASEDATA_FILENAME");

/// Panics if:
/// - In release mode, and the executable path can't be retrieved.
/// - In debug mode, and the working directory path can't be retrieved.
#[must_use]
pub fn basedata_path() -> std::path::PathBuf {
	use std::path::PathBuf;

	let path: PathBuf;

	#[cfg(not(debug_assertions))]
	{
		path = [utils::path::exe_dir(), PathBuf::from(BASEDATA_FILENAME)]
			.iter()
			.collect();
	}
	#[cfg(debug_assertions)]
	{
		path = [
			std::env::current_dir().expect("Failed to get working directory"),
			PathBuf::from("data/viletech"),
		]
		.iter()
		.collect();
	}

	path
}

/// Returns an error if the engine's base data can't be found,
/// or has been tampered with somehow since application installation.
///
/// In the debug build, this looks for `$PWD/data/viletech`, and returns `Ok`
/// simply if that directory is found.
/// In the release build, this looks for `<exec dir>/viletech.zip`, and
/// ensures it's present and matches a checksum.
pub fn basedata_is_valid() -> Result<(), BaseDataError> {
	use sha3::Digest;
	use std::io::Read;
	use BaseDataError as Error;

	let path = basedata_path();

	if !path.exists() {
		return Err(Error::Missing);
	}

	if cfg!(debug_assertions) {
		return Ok(());
	}

	let mut file = std::fs::File::open(path).map_err(Error::ReadFailure)?;
	let file_len = file.metadata().map_err(Error::ReadFailure)?.len() as usize;
	let mut zip_bytes = Vec::with_capacity(file_len);
	file.read_to_end(&mut zip_bytes)
		.map_err(Error::ReadFailure)?;

	let mut hasher = sha3::Sha3_256::new();
	hasher.update(&zip_bytes[..]);
	let checksum = hasher.finalize();
	let mut string = String::with_capacity(checksum.len());

	for byte in checksum {
		string.push_str(&byte.to_string());
	}

	#[cfg_attr(debug_assertions, allow(clippy::comparison_to_empty))]
	if string == env!("BASEDATA_CHECKSUM") {
		Ok(())
	} else {
		Err(Error::ChecksumMismatch)
	}
}

#[derive(Debug)]
pub enum BaseDataError {
	Missing,
	ReadFailure(std::io::Error),
	ChecksumMismatch,
}

impl std::error::Error for BaseDataError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::ReadFailure(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for BaseDataError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let path = basedata_path();
		let p = path.display();

		match self {
			Self::Missing => write!(f, "Engine base data not found at `{p}`."),
			Self::ReadFailure(err) => err.fmt(f),
			Self::ChecksumMismatch => write!(f, "Engine base data at `{p}` is corrupted."),
		}
	}
}
