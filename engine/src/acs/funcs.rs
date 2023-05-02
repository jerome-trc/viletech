//! Function indices.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Function {
	GetLineUdmfInt = 1,
	GetLineUdmfFixed,
	GetThingUdmfInt,
	GetThingUdmfFixed,
	GetSectorUdmfInt,
	GetSectorUdmfFixed,
	GetSideUdmfInt,
	GetSideUdmfFixed,
	GetEntityVelX,
	GetEntityVelY,
	GetEntityVelZ,
	SetActivator,
	SetActivatorToTarget,
	GetEntityViewHeight,
	GetChar,
	GetAirSupply,
	SetAirSupply,
	SetSkyScrollSpeed,
	GetArmorType,
	SpawnSpotForced,
	SpawnSpotFacingForced,
	CheckEntityProperty,
	SetEntityVelocity,
	SetUserVariable,
	GetUserVariable,
	RadiusQuake2,
	CheckEntityClass,
	SetUserArray,
	GetUserArray,
	SoundSequenceOnEntity,
	SoundSequenceOnSector,
	SoundSequenceOnPolyobj,
	GetPolyobjX,
	GetPolyobjY,
	CheckSight,
	SpawnForced,
	AnnouncerSound,
	SetPointer,
	NamedExecute,
	NamedSuspend,
	NamedTerminate,
	NamedLockedExecute,
	NamedLockedExecuteDoor,
	NamedExecuteWithResult,
	NamedExecuteAlways,
	UniqueTid,
	IsTidUsed,
	Sqrt,
	FixedSqrt,
	VectorLength,
	SetHudClipRect,
	SetHudWrapWidth,
	SetCVar,
	GetUserCVar,
	SetUserCVar,
	GetCVarString,
	SetCVarString,
	GetUserCVarString,
	SetUserCVarString,
	LineAttack,
	PlaySound,
	StopSound,
	StrCmp,
	StriCmp,
	StrLeft,
	StrRight,
	StrMid,
	GetEntityClass,
	GetWeapon,
	SoundVolume,
	PlayEntitySound,
	SpawnDecal,
	CheckFont,
	DropItem,
	CheckFlag,
	SetLineActivation,
	GetLineActivation,
	GetEntityPowerupTics,
	ChangeEntityAngle,
	ChangeEntityPitch,
	GetArmorInfo,
	DropInventory,
	PickEntity,
	IsPointerEqual,
	CanRaiseEntity,
	SetEntityTeleFog,
	SwapEntityTeleFog,
	SetEntityRoll,
	ChangeEntityRoll,
	GetEntityRoll,
	QuakeEx,
	Warp,
	GetMaxInventory,
	SetSectorDamage,
	SetSectorTerrain,
	SpawnParticle,
	SetMusicVolume,
	CheckProximity,
	CheckEntityState,

	// Zandronum
	// 100 : ResetMap(0),
	// 101 : PlayerIsSpectator(1),
	// 102 : ConsolePlayerNumber(0),
	// 103 : GetTeamProperty(2),
	// 104 : GetPlayerLivesLeft(1),
	// 105 : SetPlayerLivesLeft(2),
	// 106 : KickFromGame(2),
	CheckClass = 200,
	DamageEntity,
	SetEntityFlag,
	SetTranslation,
	GetEntityFloorTexture,
	GetEntityFloorTerrain,
	StrArg,
	Floor,
	Round,
	Ceil,
	ScriptCall,
	StartSlideshow,
	GetSectorHealth,
	GetLineHealth,
	SetSubtitleNumber,
	// Eternity Engne
	GetLineX = 300,
	GetLineY,
	// Hardware renderer
	SetSectorGlow = 400,
	SetFogDensity,
	// ZDaemon
	GetTeamScore = 19620, // (int team)
	SetTeamScore,         // (int team, int value
}
