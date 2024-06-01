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

#include "viletech.nim.h"

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

#endif // __P_ENEMY__
