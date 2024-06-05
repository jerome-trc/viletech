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
 *  Sprite animation.
 *
 *-----------------------------------------------------------------------------*/

#ifndef __P_PSPR__
#define __P_PSPR__

/* Basic data types.
 * Needs fixed point, and BAM angles. */

#include "doomdef.h"
#include "m_fixed.h"
#include "tables.h"
#include "p_mobj.h"


/* Needs to include the precompiled sprite animation tables.
 *
 * Header generated by multigen utility.
 * This includes all the data for thing animation,
 * i.e. the Thing Atrributes table and the Frame Sequence table.
 */

#include "info.h"

/*
 * Frame flags:
 * handles maximum brightness (torches, muzzle flare, light sources)
 */

#define FF_FULLBRIGHT   0x8000  /* flag in thing->frame */
#define FF_FRAMEMASK    0x7fff

/*
 * Overlay psprites are scaled shapes
 * drawn directly on the view screen,
 * coordinates are given for a 320*200 view screen.
 */

typedef enum
{
  ps_weapon,
  ps_flash,
  NUMPSPRITES
} psprnum_t;

typedef struct
{
  state_t *state;       /* a NULL state means not active */
  int     tics;
  fixed_t sx;
  fixed_t sy;
} pspdef_t;

typedef void (*PSprAction)(CCore*, struct player_s*, pspdef_t*);

enum
{
    CENTERWEAPON_OFF,
    CENTERWEAPON_HOR,
    CENTERWEAPON_HORVER,
    CENTERWEAPON_BOB,
    NUM_CENTERWEAPON,
};

int P_WeaponPreferred(int w1, int w2);

int P_SwitchWeapon(struct player_s *player);
dboolean P_CheckAmmo(CCore*, struct player_s *player);
void P_SubtractAmmo(struct player_s *player, int compat_amt);
void P_SetupPsprites(CCore*, struct player_s *curplayer);
void P_MovePsprites(CCore*, struct player_s *curplayer);
void P_DropWeapon(CCore*, struct player_s *player);
int P_AmmoPercent(struct player_s *player, int weapon);

void A_BFGSpray(CCore*, struct player_s*, mobj_t*);

void A_BFGsound(CCore*, struct player_s*, pspdef_t*);
void A_CheckReload(CCore*, struct player_s*, pspdef_t*);
void A_CloseShotgun2(CCore*, struct player_s*, pspdef_t*);
void A_FireBFG(CCore*, struct player_s*, pspdef_t*);
void A_FireCGun(CCore*, struct player_s*, pspdef_t*);
void A_FireMissile(CCore*, struct player_s*, pspdef_t*);
void A_FireOldBFG(CCore*, struct player_s*, pspdef_t*);
void A_FirePistol(CCore*, struct player_s*, pspdef_t*);
void A_FirePlasma(CCore*, struct player_s*, pspdef_t*);
void A_FireShotgun(CCore*, struct player_s*, pspdef_t*);
void A_FireShotgun2(CCore*, struct player_s*, pspdef_t*);
void A_GunFlash(CCore*, struct player_s*, pspdef_t*);
void A_Light0(CCore*, struct player_s*, pspdef_t*);
void A_Light1(CCore*, struct player_s*, pspdef_t*);
void A_Light2(CCore*, struct player_s*, pspdef_t*);
void A_LoadShotgun2(CCore*, struct player_s*, pspdef_t*);
void A_Lower(CCore*, struct player_s*, pspdef_t*);
void A_OpenShotgun2(CCore*, struct player_s*, pspdef_t*);
void A_Punch(CCore*, struct player_s*, pspdef_t*);
void A_Raise(CCore*, struct player_s*, pspdef_t*);
void A_ReFire(CCore*, struct player_s*, pspdef_t*);
void A_Saw(CCore*, struct player_s*, pspdef_t*);
void A_WeaponReady(CCore*, struct player_s*, pspdef_t*);

// [XA] New mbf21 codepointers

void A_WeaponProjectile(CCore*, struct player_s*, pspdef_t*);
void A_WeaponBulletAttack(CCore*, struct player_s*, pspdef_t*);
void A_WeaponMeleeAttack(CCore*, struct player_s*, pspdef_t*);
void A_WeaponSound(CCore*, struct player_s*, pspdef_t*);
void A_WeaponAlert(CCore*, struct player_s*, pspdef_t*);
void A_WeaponJump(CCore*, struct player_s*, pspdef_t*);
void A_ConsumeAmmo(CCore*, struct player_s*, pspdef_t*);
void A_CheckAmmo(CCore*, struct player_s*, pspdef_t*);
void A_RefireTo(CCore*, struct player_s*, pspdef_t*);
void A_GunFlashTo(CCore*, struct player_s*, pspdef_t*);

// heretic

void P_RepositionMace(mobj_t * mo);
void P_ActivateBeak(CCore*, struct player_s * player);
void P_PostChickenWeapon(CCore*, struct player_s *, weapontype_t);
void P_SetPsprite(CCore*, struct player_s *, int position, statenum_t);
void P_SetPspritePtr(CCore*, struct player_s*, pspdef_t*, statenum_t);
void P_OpenWeapons(void);
void P_CloseWeapons(void);
void P_AddMaceSpot(const mapthing_t * mthing);
void P_UpdateBeak(struct player_s*, pspdef_t*);

// hexen

void P_SetPspriteNF(struct player_s*, int position, statenum_t);
void P_PostMorphWeapon(CCore*, struct player_s*, weapontype_t);
void P_ActivateMorphWeapon(CCore*, struct player_s*);

#endif
