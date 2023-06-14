use bevy::prelude::*;
use crossbeam::channel::Sender;
use std::{
	path::{Path, PathBuf},
	time::Instant,
};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_log::LogTracer;
use tracing_subscriber::{
	fmt::{format::Writer, time::FormatTime},
	prelude::*,
	EnvFilter,
};

use crate::{
	console, short_version_string,
	util::{self, duration_to_hhmmss},
};

pub use tracing::Level;

#[derive(Debug)]
pub struct TracingPlugin {
	/// Filters logs using the [`tracing_subscriber::EnvFilter`] format.
	pub filter: String,
	/// Filters out logs that are "less than" the given level.
	/// This can be further filtered using the `filter` setting.
	pub level: tracing::Level,
	pub console_sender: Option<Sender<console::Message>>,
}

impl TracingPlugin {
	fn filter_layer(&self) -> EnvFilter {
		let default_filter = format!("{},{}", self.level, self.filter);

		EnvFilter::try_from_default_env()
			.or_else(|_| EnvFilter::try_new(&default_filter))
			.unwrap()
	}

	fn file_writer() -> (NonBlocking, WorkerGuard) {
		let exe_dir = util::path::exe_dir();

		let fpath: PathBuf = [&exe_dir, Path::new("viletech.log")].iter().collect();

		if fpath.exists() {
			let oldpath: PathBuf = [&exe_dir, Path::new("viletech.log.old")].iter().collect();

			if let Err(err) = std::fs::rename(&fpath, oldpath) {
				eprintln!("Failed to rotate previous log file: {err}");
			}
		}

		let appender = tracing_appender::rolling::never(&exe_dir, "viletech.log");
		tracing_appender::non_blocking(appender)
	}
}

impl Default for TracingPlugin {
	fn default() -> Self {
		Self {
			filter: "wgpu=error".to_string(),
			level: tracing::Level::INFO,
			console_sender: None,
		}
	}
}

impl bevy::prelude::Plugin for TracingPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		use tracing_subscriber::fmt;

		#[derive(Debug, Resource)]
		struct FileAppenderWorkerGuard(WorkerGuard);

		#[derive(Debug, Resource)]
		struct ConsoleWriterWorkerGuard(WorkerGuard);

		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		struct Uptime(Instant);

		impl FormatTime for Uptime {
			fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
				let (hh, mm, ss) = duration_to_hhmmss(self.0.elapsed());
				write!(w, "{hh:02}:{mm:02}:{ss:02}")
			}
		}

		let start_time = crate::START_TIME
			.get()
			.expect("`viletech::START_TIME` must be set to use `TracingPlugin`");

		let logger_set = LogTracer::init().is_err();

		let (fwriter, guard) = Self::file_writer();

		app.insert_resource(FileAppenderWorkerGuard(guard));

		let layer_stdout = fmt::Layer::default().with_timer(Uptime(*start_time));

		let layer_file = fmt::Layer::default()
			.with_ansi(false)
			.with_target(false)
			.with_timer(Uptime(*start_time))
			.with_writer(fwriter);

		let subscriber_set = if let Some(sender) = &self.console_sender {
			let writer = console::Writer::new(sender.clone());
			let (cwriter, guard) = tracing_appender::non_blocking(writer);

			app.insert_resource(ConsoleWriterWorkerGuard(guard));

			let layer_console = fmt::Layer::default()
				.with_ansi(false)
				.with_target(false)
				.with_timer(Uptime(*start_time))
				.with_writer(cwriter);

			let collector = tracing_subscriber::registry()
				.with(self.filter_layer())
				.with(layer_stdout.and_then(layer_file).and_then(layer_console));

			tracing::subscriber::set_global_default(collector).is_err()
		} else {
			unimplemented!()
		};

		match (logger_set, subscriber_set) {
			(true, true) => warn!(
				"Could not set global logger and tracing subscriber as they are already set. \
				Consider disabling `LogPlugin`."
			),
			(true, _) => warn!(
				"Could not set global logger as it is already set. \
			Consider disabling `LogPlugin`."
			),
			(_, true) => warn!(
				"Could not set global tracing subscriber as it is already set. \
			Consider disabling `LogPlugin`."
			),
			_ => (),
		};
	}
}

/// After [`TracingPlugin`] has been built, call this to print:
/// - The engine's semantic version.
/// - The application's semantic version.
/// - Information about the operating system.
/// - Information about the CPU's relevant properties.
pub fn init_diag(app_version_string: &str) -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
	fn log_cpu_info() -> Result<(), Box<dyn std::error::Error>> {
		unimplemented!("CPU diagnostics logging only available on x86(_64)");
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
				"\t- Family ID: {} ({} base, {} extended)\r\n\
				\t- Model ID: {} ({} base, {} extended)\r\n\
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

		info!("CPU diagnostics: \r\n{}", output);

		Ok(())
	}

	info!("{}", short_version_string());
	info!("{}", app_version_string);
	info!("{}", os_info()?);
	log_cpu_info()?;

	Ok(())
}

fn os_info() -> Result<String, Box<dyn std::error::Error>> {
	type Command = std::process::Command;

	match std::env::consts::OS {
		"linux" => {
			let uname = Command::new("uname").args(["-s", "-r", "-v"]).output();

			let output = match uname {
				Ok(o) => o,
				Err(err) => {
					error!("Failed to execute `uname -s -r -v`: {}", err);
					return Err(Box::new(err));
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s.replace('\n', ""),
				Err(err) => {
					error!(
						"Failed to convert `uname -s -r -v` output to UTF-8: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			Ok(osinfo)
		}
		"windows" => {
			let systeminfo = Command::new("systeminfo | findstr")
				.args(["/C:\"OS\""])
				.output();

			let output = match systeminfo {
				Ok(o) => o,
				Err(err) => {
					error!(
						"Failed to execute `systeminfo | findstr /C:\"OS\"`: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s,
				Err(err) => {
					error!(
						"Failed to convert `systeminfo | findstr /C:\"OS\"` \
						 output to UTF-8: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			Ok(osinfo)
		}
		_ => Err(Box::<std::io::Error>::new(
			std::io::ErrorKind::Unsupported.into(),
		)),
	}
}
