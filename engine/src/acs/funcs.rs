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
along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

pub(super) enum Function {
	GetLineUdmfInt = 1,
	GetLineUdmfFixed,
	GetThingUdmfInt,
	GetThingUdmfFixed,
	GetSectorUdmfInt,
	GetSectorUdmfFixed,
	GetSideUdmfInt,
	GetSideUdmfFixed,
	GetActorVelX,
	GetActorVelY,
	GetActorVelZ,
	SetActivator,
	SetActivatorToTarget,
	GetActorViewHeight,
	GetChar,
	GetAirSupply,
	SetAirSupply,
	SetSkyScrollSpeed,
	GetArmorType,
	SpawnSpotForced,
	SpawnSpotFacingForced,
	CheckActorProperty,
	SetActorVelocity,
	SetUserVariable,
	GetUserVariable,
	RadiusQuake2,
	CheckActorClass,
	SetUserArray,
	GetUserArray,
	SoundSequenceOnActor,
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
	GetActorClass,
	GetWeapon,
	SoundVolume,
	PlayActorSound,
	SpawnDecal,
	CheckFont,
	DropItem,
	CheckFlag,
	SetLineActivation,
	GetLineActivation,
	GetActorPowerupTics,
	ChangeActorAngle,
	ChangeActorPitch,
	GetArmorInfo,
	DropInventory,
	PickActor,
	IsPointerEqual,
	CanRaiseActor,
	SetActorTeleFog,
	SwapActorTeleFog,
	SetActorRoll,
	ChangeActorRoll,
	GetActorRoll,
	QuakeEx,
	Warp,
	GetMaxInventory,
	SetSectorDamage,
	SetSectorTerrain,
	SpawnParticle,
	SetMusicVolume,
	CheckProximity,
	CheckActorState,

	// Zandronum
	// 100 : ResetMap(0),
	// 101 : PlayerIsSpectator(1),
	// 102 : ConsolePlayerNumber(0),
	// 103 : GetTeamProperty(2),
	// 104 : GetPlayerLivesLeft(1),
	// 105 : SetPlayerLivesLeft(2),
	// 106 : KickFromGame(2),
	CheckClass = 200,
	DamageActor,
	SetActorFlag,
	SetTranslation,
	GetActorFloorTexture,
	GetActorFloorTerrain,
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
