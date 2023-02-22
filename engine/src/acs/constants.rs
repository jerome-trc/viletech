//! Constants used by scripts themselves, not by the engine.

/*
** -----------------------------------------------------------------------------
** Copyright 1998-2012 Randy Heit
** All rights reserved.
**
** Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions
** are met:
**
** 1. Redistributions of source code must retain the above copyright
**    notice, this list of conditions and the following disclaimer.
** 2. Redistributions in binary form must reproduce the above copyright
**    notice, this list of conditions and the following disclaimer in the
**    documentation and/or other materials provided with the distribution.
** 3. The name of the author may not be used to endorse or promote products
**    derived from this software without specific prior written permission.
**
** THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
** IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
** OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
** IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
** INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
** NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
** THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
** -----------------------------------------------------------------------------
*/

use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
	FriendlySeeBlocks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ArmorInfo {
	ClassName,
	SaveAmount,
	SavePercent,
	MaxAbsorb,
	MaxFullAbsorb,
	ActualSaveAmount,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum BlockType {
	Nothing,
	Creatures,
	Everything,
	Railing,
	Players,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum GameScene {
	Single,
	NetCoop,
	NetDeathmatch,
	Titlemap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PrintName {
	LevelName = -1,
	Level = -2,
	Skill = -3,
	NextLevel = -4,
	NextSecret = -5,
}

pub(super) const SIDE_FRONT: i32 = 0;
pub(super) const SIDE_BACK: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Skill {
	VeryEasy,
	Easy,
	Normal,
	Hard,
	VeryHard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Texture {
	Top,
	Middle,
	Bottom,
}
