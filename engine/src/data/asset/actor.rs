//! The [actor blueprint](Blueprint) and related symbols.

use bitflags::bitflags;

use crate::{data::InHandle, sim::actor, EditorNum, SpawnNum};

use super::{AssetHeader, Audio, Image, PolyModel, VoxelModel};

/// The prototype used to instantiate new entities.
///
/// Maps to a VZScript class, which are used to define these.
#[derive(Debug)]
pub struct Blueprint {
	pub header: AssetHeader,
	pub base: Components,
	pub blood_types: [Option<InHandle<Blueprint>>; 3],
	pub bones: Option<()>, // TODO: Skeletal animation w/ data representation.
	pub damage_factors: Vec<(InHandle<DamageType>, f64)>,
	/// Actor is shrunk to this height after being killed.
	pub death_height: f64,
	/// Actor is shrunk to this height after a burning death.
	pub burn_height: f64,
	pub fsm: FStateMachine,
	/// Default is 1000. What the actor's health gets set to upon spawning.
	pub health_starting: i32,
	/// Health value below which actor enters "extreme death" f-state sequence.
	pub gib_health: i32,
	pub model: Option<()>, // TODO: Polymodel data representation.
	/// What to write if a player actor was killed by this actor's non-melee attack.
	pub obituary: String, // TODO: String interning.
	/// What to write if a player actor was killed by this actor's melee attack.
	pub obituary_melee: String, // TODO: String interning.
	pub pain_chances: Vec<(InHandle<DamageType>, u16)>,
	pub render_feats: RenderFeatures,
	pub sounds: Sounds,
	pub editor_num: EditorNum,
	pub spawn_num: SpawnNum,
	// TODO:
	// - Finite state machine definition
	// - VileTech's replacement system
	// - Conversation ID (Strife)
	// - Infighting group
	// - Projectile group
	// - Light assocations
	// - Loot drop table
	// - Splash group (?)
	// - `SkipSuperSet` (?)
	// Q: Do we want or need IWAD filtering?
}

/// Used only for composing [`Blueprint`].
#[derive(Debug)]
pub struct Components {
	pub monster: Option<actor::Monster>,
	pub projectile: Option<actor::Projectile>,
}

/// "Finite state machine". See [`FState`] to learn more.
/// Sub-structure used to compose [`Blueprint`]. Mostly exists for cleanliness.
#[derive(Debug)]
pub struct FStateMachine {
	/// Each element's field `::1` indexes into `states`.
	pub labels: Vec<(String, usize)>,
	/// Ordering is defined by the script-based blueprint.
	pub states: Vec<FState>,
}

/// "Finite state". An actor appearance tied to some behavior and a tick duration.
/// See [`FStateMachine`].
pub struct FState {
	pub visual: FStateVisual,
	pub duration: i16,
	pub tick_range: u16,
	pub flags: FStateFlags,
	// TODO: What params/return types?
	pub action: Option<Box<dyn FnMut() + Send + Sync + 'static>>,
}

impl std::fmt::Debug for FState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FState")
			.field("visual", &self.visual)
			.field("duration", &self.duration)
			.field("tick_range", &self.tick_range)
			.field("flags", &self.flags)
			.field(
				"action",
				&self
					.action
					.as_ref()
					.map(|_| "<dyn FnMut() + Send + Sync>")
					.unwrap_or("<null>"),
			)
			.finish()
	}
}

impl FState {
	#[must_use]
	pub fn infinite(&self) -> bool {
		self.duration == -1
	}
}

bitflags! {
	/// See [`FState`].
	pub struct FStateFlags: u8 {
		const FAST = 1 << 0;
		const SLOW = 1 << 1;
		const FULLBRIGHT = 1 << 2;
		const CAN_RAISE = 1 << 3;
		const USER_0 = 1 << 4;
		const USER_1 = 1 << 5;
		const USER_2 = 1 << 6;
		const USER_3 = 1 << 7;
	}
}

/// See [`FState`].
#[derive(Debug)]
pub enum FStateVisual {
	/// Known commonly as `TNT1`.
	None,
	Sprite(InHandle<Image>),
	Voxel(InHandle<VoxelModel>),
	Poly(InHandle<PolyModel>),
}

/// Sub-structure used to compose [`Blueprint`]. Exists only for cleanliness.
#[derive(Debug)]
pub struct Sounds {
	/// Played when the actor is electrocuted or poisoned.
	pub howl: Option<InHandle<Audio>>,
	pub melee: Option<InHandle<Audio>>,
	pub rip: Option<InHandle<Audio>>,
	/// Played when the actor is being pushed by something.
	pub push: Option<InHandle<Audio>>,
}

bitflags! {
	/// Flags for filtering actor visibility based on capabilities of the
	/// currently-used renderer.
	pub struct RenderFeatures: u16 {
		const FLAT_SPRITES = 1 << 0;
		const MODELS = 1 << 1;
		const SLOPE_3D_FLOORS = 1 << 2;
		/// i.e. full vertical free-look.
		const TILT_PITCH = 1 << 3;
		const ROLL_SPRITES = 1 << 4;
		/// Mid-textures and sprites can render "into" flats and walls.
		const UNCLIPPED_TEX = 1 << 5;
		/// "Material shaders".
		const MAT_SHADER = 1 << 6;
		/// Post-process shaders (i.e. render buffers).
		const POST_SHADER = 1 << 7;
		const BRIGHTMAP = 1 << 8;
		/// Custom colormaps, including the ability to fullbright certain ranges,
		/// a la Strife.
		const COLORMAP = 1 << 9;
		/// Uses polygons instead of wallscans/visplanes
		/// (i.e. softpoly and hardware).
		const POLYGONAL = 1 << 10;
		const TRUECOLOR = 1 << 11;
		const VOXELS = 1 << 12;
	}
}

impl std::ops::Deref for Blueprint {
	type Target = Components;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

impl std::ops::DerefMut for Blueprint {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.base
	}
}

#[derive(Debug)]
pub struct DamageType {
	pub header: AssetHeader,
	pub base_factor: f32,
	pub flags: DamageTypeFlags,
}

bitflags! {
	pub struct DamageTypeFlags: u8 {
		const REPLACE_FACTOR = 1 << 0;
		const BYPASS_ARMOR = 1 << 1;
	}
}

#[derive(Debug)]
pub struct Species {
	pub header: AssetHeader,
	// ???
}
