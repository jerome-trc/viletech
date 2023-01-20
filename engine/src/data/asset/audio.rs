//! Sounds and music assets.

use super::Asset;

#[derive(Debug)]
pub struct Music {
	pub kind: MusicKind,
	// Q: Metadata?
}

#[derive(Debug)]
pub enum MusicKind {
	Midi, // TODO: Populate when ZMusic gets replaced
	Waveform,
}

impl Asset for Music {}
