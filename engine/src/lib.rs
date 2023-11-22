//! # VileTech Engine
//!
//! VileTech is a pet project, meant as an experiment in creating a modern,
//! feature-oriented Doom source port in Rust which can successfully interpret
//! user-generated content for GZDoom and the Eternity Engine with zero end-user
//! overhead and minimal runtime overhead.

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

pub mod asset;
pub mod audio;
pub mod basedata;
pub extern crate bytemuck;
// pub mod catalog;
pub mod console;
pub extern crate crossbeam;
pub extern crate dashmap;
pub extern crate data;
pub mod frontend;
pub mod gfx;
pub extern crate image;
pub extern crate indexmap;
pub extern crate kira;
pub use data::level;
pub extern crate lith;
pub mod log;
pub extern crate mus;
pub extern crate nanorand;
pub mod player;
pub extern crate rayon;
pub extern crate regex;
pub mod rng;
pub extern crate rustc_hash;
// pub mod sim;
pub mod terminal;
pub extern crate tracing;
pub mod types;
pub mod user;
pub extern crate util;
pub extern crate vfs;
pub extern crate wadload;
pub mod world;

// Types ///////////////////////////////////////////////////////////////////////

/// See [`bevy::render::color::Color::Rgba`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaF32 {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
	pub alpha: f32,
}

/// Type alias for Bevy's two-point rectangle to disambiguate from VileTech's [4-point counterpart].
///
/// [4-point counterpart]: util::math::Rect4
pub type Rect2 = bevy::math::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BaseGame {
	Doom,
	Hexen,
	Heretic,
	Strife,
	ChexQuest,
}

/// Newtype around [`vfs::VirtualFs`] which implements [`Resource`].
///
/// [`Resource`]: bevy::ecs::system::system_param::Resource
#[derive(bevy::prelude::Resource, bevy::prelude::Deref, bevy::prelude::DerefMut)]
pub struct VirtualFs(pub vfs::VirtualFs);

// Constants ///////////////////////////////////////////////////////////////////

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const COMPILE_DATETIME: &str = env!("COMPILE_DATETIME");

// Symbols that don't belong in any other module ///////////////////////////////

/// Ideally setting this is the first operation your application makes.
/// Used by the [`log::TracingPlugin`] for formatting time in log messages.
pub static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Prepares the rayon global thread pool. See [`rayon::ThreadPoolBuilder`].
/// If `num_threads` is `None` then rayon chooses it automatically.
/// This also ensures that these threads have clear names for debugging purposes.
pub fn thread_pool_init(num_threads: Option<usize>) {
	rayon::ThreadPoolBuilder::new()
		.thread_name(|index| format!("vile-global-{index}"))
		.num_threads(num_threads.unwrap_or(0))
		.build_global()
		.expect("failed to build Rayon's global thread pool")
}

/// Returns a message telling the user the engine's `CARGO_PKG_VERSION`.
#[must_use]
pub fn short_version_string() -> String {
	format!("VileTech Engine {}", env!("CARGO_PKG_VERSION"))
}

#[must_use]
pub fn version_info() -> [String; 3] {
	[
		short_version_string(),
		format!("Commit {GIT_HASH}"),
		format!("Compiled on {COMPILE_DATETIME}"),
	]
}

/// Allows upcasting from `dyn T` to [`std::any::Any`].
pub trait AsAny: std::any::Any {
	#[must_use]
	fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
	fn as_any(&self) -> &dyn std::any::Any {
		// (RAT) As silly as this seems, it is the only halfway-elegant way of
		// doing this, and it has some surprisingly useful niche applications.
		self
	}
}
