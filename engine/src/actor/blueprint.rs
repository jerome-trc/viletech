use bevy::prelude::*;
use data::{EditorNum, SpawnNum};

/// The prototype used to instantiate new entities.
#[derive(Asset, TypePath, Debug)]
pub struct Blueprint {
	pub editor_num: EditorNum,
	pub spawn_num: SpawnNum,
}
