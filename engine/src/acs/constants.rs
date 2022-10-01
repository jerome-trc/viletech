//! Constants used by scripts themselves, not by the engine.

/*
Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use bitflags::bitflags;

pub(super) enum ActorProperty {
	Health,
	Speed,
	Damage,
	Alpha,
	RenderStyle,
	SeeSound,
	AttackSound,
	PainSound,
	DeathSound,
	ActiveSound,
	Ambush,
	Invulnerable,
	JumpZ,
	ChaseGoal,
	Frightened,
	Gravity,
	Friendly,
	SpawnHealth,
	Dropped,
	Notarget,
	Species,
	NameTag,
	Score,
	Notrigger,
	DamageFactor,
	MasterTID,
	TargetTID,
	TracerTID,
	WaterLevel,
	ScaleX,
	ScaleY,
	Dormant,
	Mass,
	Accuracy,
	Stamina,
	Height,
	Radius,
	ReactionTime,
	MeleeRange,
	ViewHeight,
	AttackZOffset,
	StencilColor,
	Friction,
	DamageMultiplier,
	MaxStepHeight,
	MaxDropOffHeight,
	DamageType,
	SoundClass,
}

pub(super) enum ArmorInfo {
	ClassName,
	SaveAmount,
	SavePercent,
	MaxAbsorb,
	MaxFullAbsorb,
	ActualSaveAmount,
}

#[repr(i32)]
pub(super) enum BlockType {
	Nothing,
	Creatures,
	Everything,
	Railing,
	Players,
}

pub(super) enum GameScene {
	Single,
	NetCoop,
	NetDeathmatch,
	Titlemap,
}

pub(super) enum HexenClass {
	Fighter,
	Cleric,
	Mage,
}

bitflags! {
	pub(super) struct HudMessageFlags: u8 {
		const COLOR_STRING = 1 << 0;
		const ADD_BLEND = 1 << 1;
		const ALPHA = 1 << 2;
		const NO_WRAP = 1 << 3;
	}
}

#[repr(i32)]
pub(super) enum LevelInfo {
	ParTime,
	ClusterNum,
	LevelNum,
	TotalSecrets,
	FoundSecrets,
	TotalItems,
	FoundItems,
	TotalMonsters,
	KilledMonsters,
	SuckTime,
}

bitflags! {
	pub(super) struct LineAttackFlags: u8 {
		const NO_RANDOM_PUFF_Z = 1 << 0;
		const NO_IMPACT_DECAL = 1 << 1;
	}
}

pub(super) const LINE_FRONT: i32 = 0;
pub(super) const LINE_BACK: i32 = 1;

bitflags! {
	pub(super) struct PickActorFlags: u8 {
		const FORCE_TID = 1 << 0;
		const RETURN_TID = 1 << 1;
	}
}

#[repr(i32)]
pub(super) enum PlayerInfo {
	Team,
	AimDist,
	Color,
	Gender,
	NeverSwitch,
	MoveBob,
	StillBob,
	PlayerClass,
	Fov,
	DesiredFov,
}

#[repr(i8)]
pub(super) enum PrintName {
	LevelName = -1,
	Level = -2,
	Skill = -3,
	NextLevel = -4,
	NextSecret = -5,
}

pub(super) const SIDE_FRONT: i32 = 0;
pub(super) const SIDE_BACK: i32 = 1;

pub(super) enum Skill {
	VeryEasy,
	Easy,
	Normal,
	Hard,
	VeryHard,
}

pub(super) enum SoundType {
	See,
	Attack,
	Pain,
	Death,
	Active,
	Use,
	Bounce,
	WallBounce,
	CrushPain,
	Howl,
}

bitflags! {
	pub(super) struct SpawnDecalFlags: u8 {
		const ABS_ANGLE = 1 << 0;
		const PERMANENT = 1 << 1;
		const FIXED_Z_OFFS = 1 << 2;
		const FIXED_DIST = 1 << 3;
	}
}

pub(super) enum Texture {
	Top,
	Middle,
	Bottom,
}
