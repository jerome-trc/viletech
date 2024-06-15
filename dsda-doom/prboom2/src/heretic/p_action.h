//
// Copyright(C) 1993-1996 Id Software, Inc.
// Copyright(C) 1993-2008 Raven Software
// Copyright(C) 2005-2014 Simon Howard
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
//
// External definitions for action pointer functions.
//

#ifndef HERETIC_P_ACTION_H
#define HERETIC_P_ACTION_H

#include "p_mobj.h"

// in doom

void A_Scream(CCore*, mobj_t*);
void A_Explode(CCore*, mobj_t*);
void A_Light0(CCore*, mobj_t*);
void A_WeaponReady(CCore*, mobj_t*);
void A_Lower(CCore*, mobj_t*);
void A_Raise(CCore*, mobj_t*);
void A_ReFire(CCore*, mobj_t*);
void A_Pain(CCore*, mobj_t*);
void A_SkullPop(CCore*, mobj_t*);
void A_FaceTarget(CCore*, mobj_t*);
void A_Look(CCore*, mobj_t*);
void A_Chase(CCore*, mobj_t*);
void A_HeadAttack(CCore*, mobj_t*);
void A_BossDeath(CCore*, mobj_t*);

// not in doom
void A_FreeTargMobj(CCore*, mobj_t*);
void A_RestoreSpecialThing1(CCore*, mobj_t*);
void A_RestoreSpecialThing2(CCore*, mobj_t*);
void A_HideThing(CCore*, mobj_t*);
void A_UnHideThing(CCore*, mobj_t*);
void A_RestoreArtifact(CCore*, mobj_t*);
void A_PodPain(CCore*, mobj_t*);
void A_RemovePod(CCore*, mobj_t*);
void A_MakePod(CCore*, mobj_t*);
void A_InitKeyGizmo(CCore*, mobj_t*);
void A_VolcanoSet(CCore*, mobj_t*);
void A_VolcanoBlast(CCore*, mobj_t*);
void A_BeastPuff(CCore*, mobj_t*);
void A_VolcBallImpact(CCore*, mobj_t*);
void A_SpawnTeleGlitter(CCore*, mobj_t*);
void A_SpawnTeleGlitter2(CCore*, mobj_t*);
void A_AccTeleGlitter(CCore*, mobj_t*);
void A_StaffAttackPL1(CCore*, mobj_t*);
void A_StaffAttackPL2(CCore*, mobj_t*);
void A_BeakReady(CCore*, mobj_t*);
void A_BeakRaise(CCore*, mobj_t*);
void A_BeakAttackPL1(CCore*, mobj_t*);
void A_BeakAttackPL2(CCore*, mobj_t*);
void A_GauntletAttack(CCore*, mobj_t*);
void A_FireBlasterPL1(CCore*, mobj_t*);
void A_FireBlasterPL2(CCore*, mobj_t*);
void A_SpawnRippers(CCore*, mobj_t*);
void A_FireMacePL1(CCore*, mobj_t*);
void A_FireMacePL2(CCore*, mobj_t*);
void A_MacePL1Check(CCore*, mobj_t*);
void A_MaceBallImpact(CCore*, mobj_t*);
void A_MaceBallImpact2(CCore*, mobj_t*);
void A_DeathBallImpact(CCore*, mobj_t*);
void A_FireSkullRodPL1(CCore*, mobj_t*);
void A_FireSkullRodPL2(CCore*, mobj_t*);
void A_SkullRodPL2Seek(CCore*, mobj_t*);
void A_AddPlayerRain(CCore*, mobj_t*);
void A_HideInCeiling(CCore*, mobj_t*);
void A_SkullRodStorm(CCore*, mobj_t*);
void A_RainImpact(CCore*, mobj_t*);
void A_FireGoldWandPL1(CCore*, mobj_t*);
void A_FireGoldWandPL2(CCore*, mobj_t*);
void A_FirePhoenixPL1(CCore*, mobj_t*);
void A_InitPhoenixPL2(CCore*, mobj_t*);
void A_FirePhoenixPL2(CCore*, mobj_t*);
void A_ShutdownPhoenixPL2(CCore*, mobj_t*);
void A_PhoenixPuff(CCore*, mobj_t*);
void A_RemovedPhoenixFunc(CCore*, mobj_t*);
void A_FlameEnd(CCore*, mobj_t*);
void A_FloatPuff(CCore*, mobj_t*);
void A_FireCrossbowPL1(CCore*, mobj_t*);
void A_FireCrossbowPL2(CCore*, mobj_t*);
void A_BoltSpark(CCore*, mobj_t*);
void A_NoBlocking(CCore*, mobj_t*);
void A_AddPlayerCorpse(CCore*, mobj_t*);
void A_FlameSnd(CCore*, mobj_t*);
void A_CheckBurnGone(CCore*, mobj_t*);
void A_CheckSkullFloor(CCore*, mobj_t*);
void A_CheckSkullDone(CCore*, mobj_t*);
void A_Feathers(CCore*, mobj_t*);
void A_ChicLook(CCore*, mobj_t*);
void A_ChicChase(CCore*, mobj_t*);
void A_ChicPain(CCore*, mobj_t*);
void A_ChicAttack(CCore*, mobj_t*);
void A_MummyAttack(CCore*, mobj_t*);
void A_MummyAttack2(CCore*, mobj_t*);
void A_MummySoul(CCore*, mobj_t*);
void A_ContMobjSound(CCore*, mobj_t*);
void A_MummyFX1Seek(CCore*, mobj_t*);
void A_BeastAttack(CCore*, mobj_t*);
void A_SnakeAttack(CCore*, mobj_t*);
void A_SnakeAttack2(CCore*, mobj_t*);
void A_HeadIceImpact(CCore*, mobj_t*);
void A_HeadFireGrow(CCore*, mobj_t*);
void A_WhirlwindSeek(CCore*, mobj_t*);
void A_ClinkAttack(CCore*, mobj_t*);
void A_WizAtk1(CCore*, mobj_t*);
void A_WizAtk2(CCore*, mobj_t*);
void A_WizAtk3(CCore*, mobj_t*);
void A_GhostOff(CCore*, mobj_t*);
void A_ImpMeAttack(CCore*, mobj_t*);
void A_ImpMsAttack(CCore*, mobj_t*);
void A_ImpMsAttack2(CCore*, mobj_t*);
void A_ImpDeath(CCore*, mobj_t*);
void A_ImpXDeath1(CCore*, mobj_t*);
void A_ImpXDeath2(CCore*, mobj_t*);
void A_ImpExplode(CCore*, mobj_t*);
void A_KnightAttack(CCore*, mobj_t*);
void A_DripBlood(CCore*, mobj_t*);
void A_Sor1Chase(CCore*, mobj_t*);
void A_Sor1Pain(CCore*, mobj_t*);
void A_Srcr1Attack(CCore*, mobj_t*);
void A_SorZap(CCore*, mobj_t*);
void A_SorcererRise(CCore*, mobj_t*);
void A_SorRise(CCore*, mobj_t*);
void A_SorSightSnd(CCore*, mobj_t*);
void A_Srcr2Decide(CCore*, mobj_t*);
void A_Srcr2Attack(CCore*, mobj_t*);
void A_Sor2DthInit(CCore*, mobj_t*);
void A_SorDSph(CCore*, mobj_t*);
void A_Sor2DthLoop(CCore*, mobj_t*);
void A_SorDExp(CCore*, mobj_t*);
void A_SorDBon(CCore*, mobj_t*);
void A_BlueSpark(CCore*, mobj_t*);
void A_GenWizard(CCore*, mobj_t*);
void A_MinotaurAtk1(CCore*, mobj_t*);
void A_MinotaurDecide(CCore*, mobj_t*);
void A_MinotaurAtk2(CCore*, mobj_t*);
void A_MinotaurAtk3(CCore*, mobj_t*);
void A_MinotaurCharge(CCore*, mobj_t*);
void A_MntrFloorFire(CCore*, mobj_t*);
void A_ESound(CCore*, mobj_t*);

#endif
