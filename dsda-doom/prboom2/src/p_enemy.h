/* Emacs style mode select   -*- C -*-
 *-----------------------------------------------------------------------------
 *
 *
 *  PrBoom: a Doom port merged with LxDoom and LSDLDoom
 *  based on BOOM, a modified and improved DOOM engine
 *  Copyright (C) 1999 by
 *  id Software, Chi Hoang, Lee Killough, Jim Flynn, Rand Phares, Ty Halderman
 *  Copyright (C) 1999-2000 by
 *  Jess Haas, Nicolas Kalkhof, Colin Phipps, Florian Schulze
 *  Copyright 2005, 2006 by
 *  Florian Schulze, Colin Phipps, Neil Stevens, Andrey Budko
 *
 *  This program is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU General Public License
 *  as published by the Free Software Foundation; either version 2
 *  of the License, or (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA
 *  02111-1307, USA.
 *
 * DESCRIPTION:
 *      Enemy thinking, AI.
 *      Action Pointer Functions
 *      that are associated with states/frames.
 *
 *-----------------------------------------------------------------------------*/

#ifndef __P_ENEMY__
#define __P_ENEMY__

#include "p_mobj.h"

#include "viletech.zig.h"

void P_NoiseAlert (mobj_t *target, mobj_t *emmiter);
void P_SpawnBrainTargets(void); /* killough 3/26/98: spawn icon landings */
dboolean P_CheckBossDeath(mobj_t *mo);

extern struct brain_s {         /* killough 3/26/98: global state of boss brain */
  int easy, targeton;
} brain;

// ********************************************************************
// Function addresses or Code Pointers
// ********************************************************************
// These function addresses are the Code Pointers that have been
// modified for years by Dehacked enthusiasts.  The new BEX format
// allows more extensive changes (see d_deh.c)

// Doesn't work with g++, needs actionf_p1
void A_Explode(CCore*, mobj_t *);
void A_Pain(CCore*, mobj_t *);
void A_PlayerScream(CCore*, mobj_t *);
void A_Fall(CCore*, mobj_t *);
void A_XScream(CCore*, mobj_t *);
void A_Look(CCore*, mobj_t *);
void A_Chase(CCore*, mobj_t *);
void A_FaceTarget(CCore*, mobj_t *);
void A_PosAttack(CCore*, mobj_t *);
void A_Scream(CCore*, mobj_t *);
void A_SPosAttack(CCore*, mobj_t *);
void A_VileChase(CCore*, mobj_t *);
void A_VileStart(CCore*, mobj_t *);
void A_VileTarget(CCore*, mobj_t *);
void A_VileAttack(CCore*, mobj_t *);
void A_StartFire(CCore*, mobj_t *);
void A_Fire(CCore*, mobj_t *);
void A_FireCrackle(CCore*, mobj_t *);
void A_Tracer(CCore*, mobj_t *);
void A_SkelWhoosh(CCore*, mobj_t *);
void A_SkelFist(CCore*, mobj_t *);
void A_SkelMissile(CCore*, mobj_t *);
void A_FatRaise(CCore*, mobj_t *);
void A_FatAttack1(CCore*, mobj_t *);
void A_FatAttack2(CCore*, mobj_t *);
void A_FatAttack3(CCore*, mobj_t *);
void A_BossDeath(CCore*, mobj_t *);
void A_CPosAttack(CCore*, mobj_t *);
void A_CPosRefire(CCore*, mobj_t *);
void A_TroopAttack(CCore*, mobj_t *);
void A_SargAttack(CCore*, mobj_t *);
void A_HeadAttack(CCore*, mobj_t *);
void A_BruisAttack(CCore*, mobj_t *);
void A_SkullAttack(CCore*, mobj_t *);
void A_Metal(CCore*, mobj_t *);
void A_SpidRefire(CCore*, mobj_t *);
void A_BabyMetal(CCore*, mobj_t *);
void A_BspiAttack(CCore*, mobj_t *);
void A_Hoof(CCore*, mobj_t *);
void A_CyberAttack(CCore*, mobj_t *);
void A_PainAttack(CCore*, mobj_t *);
void A_PainDie(CCore*, mobj_t *);
void A_KeenDie(CCore*, mobj_t *);
void A_BrainPain(CCore*, mobj_t *);
void A_BrainScream(CCore*, mobj_t *);
void A_BrainDie(CCore*, mobj_t *);
void A_BrainAwake(CCore*, mobj_t *);
void A_BrainSpit(CCore*, mobj_t *);
void A_SpawnSound(CCore*, mobj_t *);
void A_SpawnFly(CCore*, mobj_t *);
void A_BrainExplode(CCore*, mobj_t *);
void A_Die(CCore*, mobj_t *);
void A_Detonate(CCore*, mobj_t *);        /* killough 8/9/98: detonate a bomb or other device */
void A_Mushroom(CCore*, mobj_t *);        /* killough 10/98: mushroom effect */
void A_Spawn(CCore*, mobj_t *);           // killough 11/98
void A_Turn(CCore*, mobj_t *);            // killough 11/98
void A_Face(CCore*, mobj_t *);            // killough 11/98
void A_Scratch(CCore*, mobj_t *);         // killough 11/98
void A_PlaySound(CCore*, mobj_t *);       // killough 11/98
void A_RandomJump(CCore*, mobj_t *);      // killough 11/98
void A_LineEffect(CCore*, mobj_t *);      // killough 11/98

void A_BetaSkullAttack(CCore*, mobj_t*); // killough 10/98: beta lost souls attacked different
void A_Stop(CCore*, mobj_t*);

void A_SkullPop(CCore*, mobj_t*);

// [XA] New mbf21 codepointers

void A_SpawnObject(CCore*, mobj_t*);
void A_MonsterProjectile(CCore*, mobj_t*);
void A_MonsterBulletAttack(CCore*, mobj_t*);
void A_MonsterMeleeAttack(CCore*, mobj_t*);
void A_RadiusDamage(CCore*, mobj_t*);
void A_NoiseAlert(CCore*, mobj_t*);
void A_HealChase(CCore*, mobj_t*);
void A_SeekTracer(CCore*, mobj_t*);
void A_FindTracer(CCore*, mobj_t*);
void A_ClearTracer(CCore*, mobj_t*);
void A_JumpIfHealthBelow(CCore*, mobj_t*);
void A_JumpIfTargetInSight(CCore*, mobj_t*);
void A_JumpIfTargetCloser(CCore*, mobj_t*);
void A_JumpIfTracerInSight(CCore*, mobj_t*);
void A_JumpIfTracerCloser(CCore*, mobj_t*);
void A_JumpIfFlagsSet(CCore*, mobj_t*);
void A_AddFlags(CCore*, mobj_t*);
void A_RemoveFlags(CCore*, mobj_t*);

// heretic

void A_UnHideThing(CCore*, mobj_t*);
void P_InitMonsters(void);
void P_AddBossSpot(fixed_t x, fixed_t y, angle_t angle);
void P_Massacre(CCore*);
void P_DSparilTeleport(CCore*, mobj_t*);
void Heretic_A_BossDeath(CCore*, mobj_t*);
dboolean Heretic_P_LookForMonsters(mobj_t*);
dboolean Raven_P_LookForPlayers(mobj_t*, dboolean allaround);

// hexen

void P_InitCreatureCorpseQueue(CCore*, dboolean corpseScan);
void A_DeQueueCorpse(CCore*, mobj_t*);
dboolean A_RaiseMobj(CCore*, mobj_t*);
dboolean A_SinkMobj(CCore*, mobj_t*);
void A_NoBlocking(CCore*, mobj_t*);

// zdoom

dboolean P_RaiseThing(CCore*, mobj_t *corpse, mobj_t *raiser);

// Unsorted

void A_AddPlayerCorpse(CCore*, mobj_t*);
void A_BatMove(CCore*, mobj_t*);
void A_BatSpawn(CCore*, mobj_t*);
void A_BatSpawnInit(CCore*, mobj_t*);
void A_BellReset1(CCore*, mobj_t*);
void A_BellReset2(CCore*, mobj_t*);
void A_BishopAttack(CCore*, mobj_t*);
void A_BishopAttack2(CCore*, mobj_t*);
void A_BishopChase(CCore*, mobj_t*);
void A_BishopDecide(CCore*, mobj_t*);
void A_BishopDoBlur(CCore*, mobj_t*);
void A_BishopMissileSeek(CCore*, mobj_t*);
void A_BishopMissileWeave(CCore*, mobj_t*);
void A_BishopPainBlur(CCore*, mobj_t*);
void A_BishopPuff(CCore*, mobj_t*);
void A_BishopSpawnBlur(CCore*, mobj_t*);
void A_BounceCheck(CCore*, mobj_t*);
void A_BridgeInit(CCore*, mobj_t*);
void A_BridgeOrbit(CCore*, mobj_t*);
void A_CentaurAttack(CCore*, mobj_t*);
void A_CentaurAttack2(CCore*, mobj_t*);
void A_CentaurDefend(CCore*, mobj_t*);
void A_CentaurDropStuff(CCore*, mobj_t*);
void A_CFlameMissile(CCore*, mobj_t*);
void A_CFlamePuff(CCore*, mobj_t*);
void A_CFlameRotate(CCore*, mobj_t*);
void A_Chase(CCore*, mobj_t*);
void A_CheckBurnGone(CCore*, mobj_t*);
void A_CheckFloor(CCore*, mobj_t*);
void A_CheckSkullDone(CCore*, mobj_t*);
void A_CheckSkullFloor(CCore*, mobj_t*);
void A_CheckTeleRing(CCore*, mobj_t*);
void A_CheckThrowBomb(CCore*, mobj_t*);
void A_CHolyAttack2(CCore*, mobj_t*);
void A_CHolyCheckScream(CCore*, mobj_t*);
void A_CHolySeek(CCore*, mobj_t*);
void A_CHolySpawnPuff(CCore*, mobj_t*);
void A_CHolyTail(CCore*, mobj_t*);
void A_ClassBossHealth(CCore*, mobj_t*);
void A_ClericAttack(CCore*, mobj_t*);
void A_ContMobjSound(CCore*, mobj_t*);
void A_CorpseBloodDrip(CCore*, mobj_t*);
void A_CorpseExplode(CCore*, mobj_t*);
void A_CStaffMissileSlither(CCore*, mobj_t*);
void A_DelayGib(CCore*, mobj_t*);
void A_Demon2Death(CCore*, mobj_t*);
void A_DemonAttack1(CCore*, mobj_t*);
void A_DemonAttack2(CCore*, mobj_t*);
void A_DemonDeath(CCore*, mobj_t*);
void A_DragonAttack(CCore*, mobj_t*);
void A_DragonCheckCrash(CCore*, mobj_t*);
void A_DragonFlap(CCore*, mobj_t*);
void A_DragonFlight(CCore*, mobj_t*);
void A_DragonFX2(CCore*, mobj_t*);
void A_DragonInitFlight(CCore*, mobj_t*);
void A_DragonPain(CCore*, mobj_t*);
void A_DropMace(CCore*, mobj_t*);
void A_ESound(CCore*, mobj_t*);
void A_EttinAttack(CCore*, mobj_t*);
void A_Explode(CCore*, mobj_t*);
void A_FaceTarget(CCore*, mobj_t*);
void A_FastChase(CCore*, mobj_t*);
void A_FighterAttack(CCore*, mobj_t*);
void A_FiredAttack(CCore*, mobj_t*);
void A_FiredChase(CCore*, mobj_t*);
void A_FiredRocks(CCore*, mobj_t*);
void A_FiredSplotch(CCore*, mobj_t*);
void A_FlameCheck(CCore*, mobj_t*);
void A_FloatGib(CCore*, mobj_t*);
void A_FogMove(CCore*, mobj_t*);
void A_FogSpawn(CCore*, mobj_t*);
void A_FreeTargMobj(CCore*, mobj_t*);
void A_FreezeDeath(CCore*, mobj_t*);
void A_FreezeDeathChunks(CCore*, mobj_t*);
void A_FSwordFlames(CCore*, mobj_t*);
void A_HideThing(CCore*, mobj_t*);
void A_IceCheckHeadDone(CCore*, mobj_t*);
void A_IceGuyAttack(CCore*, mobj_t*);
void A_IceGuyChase(CCore*, mobj_t*);
void A_IceGuyDie(CCore*, mobj_t*);
void A_IceGuyLook(CCore*, mobj_t*);
void A_IceGuyMissileExplode(CCore*, mobj_t*);
void A_IceGuyMissilePuff(CCore*, mobj_t*);
void A_IceSetTics(CCore*, mobj_t*);
void A_KBolt(CCore*, mobj_t*);
void A_KBoltRaise(CCore*, mobj_t*);
void A_KoraxBonePop(CCore*, mobj_t*);
void A_KoraxChase(CCore*, mobj_t*);
void A_KoraxCommand(CCore*, mobj_t*);
void A_KoraxDecide(CCore*, mobj_t*);
void A_KoraxMissile(CCore*, mobj_t*);
void A_KoraxStep(CCore*, mobj_t*);
void A_KoraxStep2(CCore*, mobj_t*);
void A_KSpiritRoam(CCore*, mobj_t*);
void A_LastZap(CCore*, mobj_t*);
void A_LeafCheck(CCore*, mobj_t*);
void A_LeafSpawn(CCore*, mobj_t*);
void A_LeafThrust(CCore*, mobj_t*);
void A_LightningClip(CCore*, mobj_t*);
void A_LightningRemove(CCore*, mobj_t*);
void A_LightningZap(CCore*, mobj_t*);
void A_Look(CCore*, mobj_t*);
void A_MageAttack(CCore*, mobj_t*);
void A_MinotaurAtk1(CCore*, mobj_t*);
void A_MinotaurAtk2(CCore*, mobj_t*);
void A_MinotaurAtk3(CCore*, mobj_t*);
void A_MinotaurCharge(CCore*, mobj_t*);
void A_MinotaurChase(CCore*, mobj_t*);
void A_MinotaurDecide(CCore*, mobj_t*);
void A_MinotaurFade0(CCore*, mobj_t*);
void A_MinotaurFade1(CCore*, mobj_t*);
void A_MinotaurFade2(CCore*, mobj_t*);
void A_MinotaurLook(CCore*, mobj_t*);
void A_MinotaurRoam(CCore*, mobj_t*);
void A_MntrFloorFire(CCore*, mobj_t*);
void A_MStaffTrack(CCore*, mobj_t*);
void A_MStaffWeave(CCore*, mobj_t*);
void A_NoBlocking(CCore*, mobj_t*);
void A_NoGravity(CCore*, mobj_t*);
void A_Pain(CCore*, mobj_t*);
void A_PigAttack(CCore*, mobj_t*);
void A_PigChase(CCore*, mobj_t*);
void A_PigLook(CCore*, mobj_t*);
void A_PigPain(CCore*, mobj_t*);
void A_PoisonBagCheck(CCore*, mobj_t*);
void A_PoisonBagDamage(CCore*, mobj_t*);
void A_PoisonBagInit(CCore*, mobj_t*);
void A_PoisonShroom(CCore*, mobj_t*);
void A_PotteryCheck(CCore*, mobj_t*);
void A_PotteryChooseBit(CCore*, mobj_t*);
void A_PotteryExplode(CCore*, mobj_t*);
void A_Quake(CCore*, mobj_t*);
void A_QueueCorpse(CCore*, mobj_t*);
void A_RestoreArtifact(CCore*, mobj_t*);
void A_RestoreSpecialThing1(CCore*, mobj_t*);
void A_RestoreSpecialThing2(CCore*, mobj_t*);
void A_Scream(CCore*, mobj_t*);
void A_SerpentBirthScream(CCore*, mobj_t*);
void A_SerpentChase(CCore*, mobj_t*);
void A_SerpentCheckForAttack(CCore*, mobj_t*);
void A_SerpentChooseAttack(CCore*, mobj_t*);
void A_SerpentDiveSound(CCore*, mobj_t*);
void A_SerpentHeadCheck(CCore*, mobj_t*);
void A_SerpentHeadPop(CCore*, mobj_t*);
void A_SerpentHide(CCore*, mobj_t*);
void A_SerpentHumpDecide(CCore*, mobj_t*);
void A_SerpentLowerHump(CCore*, mobj_t*);
void A_SerpentMeleeAttack(CCore*, mobj_t*);
void A_SerpentMissileAttack(CCore*, mobj_t*);
void A_SerpentRaiseHump(CCore*, mobj_t*);
void A_SerpentSpawnGibs(CCore*, mobj_t*);
void A_SerpentUnHide(CCore*, mobj_t*);
void A_SerpentWalk(CCore*, mobj_t*);
void A_SetAltShadow(CCore*, mobj_t*);
void A_SetReflective(CCore*, mobj_t*);
void A_SetShootable(CCore*, mobj_t*);
void A_ShedShard(CCore*, mobj_t*);
void A_SinkGib(CCore*, mobj_t*);
void A_SkullPop(CCore*, mobj_t*);
void A_SmBounce(CCore*, mobj_t*);
void A_SmokePuffExit(CCore*, mobj_t*);
void A_SoAExplode(CCore*, mobj_t*);
void A_SorcBallOrbit(CCore*, mobj_t*);
void A_SorcBallPop(CCore*, mobj_t*);
void A_SorcBossAttack(CCore*, mobj_t*);
void A_SorcererBishopEntry(CCore*, mobj_t*);
void A_SorcFX1Seek(CCore*, mobj_t*);
void A_SorcFX2Orbit(CCore*, mobj_t*);
void A_SorcFX2Split(CCore*, mobj_t*);
void A_SorcFX4Check(CCore*, mobj_t*);
void A_SorcSpinBalls(CCore*, mobj_t*);
void A_SpawnBishop(CCore*, mobj_t*);
void A_SpawnFizzle(CCore*, mobj_t*);
void A_SpeedBalls(CCore*, mobj_t*);
void A_SpeedFade(CCore*, mobj_t*);
void A_Summon(CCore*, mobj_t*);
void A_TeloSpawnA(CCore*, mobj_t*);
void A_TeloSpawnB(CCore*, mobj_t*);
void A_TeloSpawnC(CCore*, mobj_t*);
void A_TeloSpawnD(CCore*, mobj_t*);
void A_ThrustBlock(CCore*, mobj_t*);
void A_ThrustImpale(CCore*, mobj_t*);
void A_ThrustInitDn(CCore*, mobj_t*);
void A_ThrustInitUp(CCore*, mobj_t*);
void A_ThrustLower(CCore*, mobj_t*);
void A_ThrustRaise(CCore*, mobj_t*);
void A_TreeDeath(CCore*, mobj_t*);
void A_UnHideThing(CCore*, mobj_t*);
void A_UnSetInvulnerable(CCore*, mobj_t*);
void A_UnSetReflective(CCore*, mobj_t*);
void A_UnSetShootable(CCore*, mobj_t*);
void A_WraithChase(CCore*, mobj_t*);
void A_WraithFX2(CCore*, mobj_t*);
void A_WraithFX3(CCore*, mobj_t*);
void A_WraithInit(CCore*, mobj_t*);
void A_WraithLook(CCore*, mobj_t*);
void A_WraithMelee(CCore*, mobj_t*);
void A_WraithMissile(CCore*, mobj_t*);
void A_WraithRaise(CCore*, mobj_t*);
void A_WraithRaiseInit(CCore*, mobj_t*);
void A_ZapMimic(CCore*, mobj_t*);

#endif // __P_ENEMY__
