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
 *  Thing events, and dehacked specified numbers controlling them.
 *
 *-----------------------------------------------------------------------------*/

#ifndef __P_INTER__
#define __P_INTER__

#include "d_player.h"
#include "p_mobj.h"

#include "viletech/core.h"

/* Ty 03/09/98 Moved to an int in p_inter.c for deh and externalization */
#define MAXHEALTH maxhealth

#define BONUSADD        6

/* follow a player exlusively for 3 seconds */
#define BASETHRESHOLD   (100)

enum {
    adf_none = 0,
    adf_painless = 1 << 0,
};
typedef int ActorDamageFlags;

typedef struct ActorDamageParams {
    mobj_t* target;
    mobj_t* inflictor;
    mobj_t* source;
    int damage;
    ActorDamageFlags flags;
} ActorDamageParams;

dboolean P_GivePower(player_t *, int);
void P_TouchSpecialThing(CCore*, mobj_t *special, mobj_t *toucher);
void P_DamageMobj(CCore*, mobj_t* target, mobj_t* inflictor, mobj_t* source, int);
void P_DamageMobj2(CCore* cx, ActorDamageParams args);
void P_HealMobj(mobj_t*, int);
int P_PlayerHealthIncrease(int value);

/* killough 5/2/98: moved from d_deh.c, g_game.c, m_misc.c, others: */

extern int god_health;   /* Ty 03/09/98 - deh support, see also p_inter.c */
extern int idfa_armor;
extern int idfa_armor_class;
extern int idkfa_armor;
extern int idkfa_armor_class;  /* Ty - end */
/* Ty 03/13/98 - externalized initial settings for respawned player */
extern int initial_health;
extern int initial_bullets;
extern int maxhealth;
extern int maxhealthbonus;
extern int max_armor;
extern int green_armor_class;
extern int blue_armor_class;
extern int max_soul;
extern int soul_health;
extern int mega_health;
extern int bfgcells;
extern int monsters_infight; // e6y: Dehacked support - monsters infight
extern int maxammo[], clipammo[];

// heretic

#define MAXCHICKENHEALTH 30

extern int GetWeaponAmmo[NUMWEAPONS];

dboolean P_GiveBody(player_t*, int num);
void P_SetMessage(CCore*, player_t*, const char*, dboolean ultmsg);
dboolean P_GiveArtifact(player_t*, artitype_t, mobj_t*);
dboolean Heretic_P_GiveWeapon(player_t*, weapontype_t);
void P_SetDormantArtifact(CCore*, mobj_t*);
void P_HideSpecialThing(CCore*, mobj_t*);
dboolean P_ChickenMorphPlayer(CCore*, player_t*);
dboolean P_ChickenMorph(CCore*, mobj_t*);
void P_TouchWhirlwind(CCore*, mobj_t*);
void P_MinotaurSlam(CCore*, mobj_t * source, mobj_t * target);
dboolean P_AutoUseChaosDevice(CCore*, player_t*);
void P_AutoUseHealth(player_t*, int saveHealth);

// hexen

#define MAXMORPHHEALTH 30

void P_SetYellowMessage(CCore*, player_t*, const char*, dboolean ultmsg);
void P_FallingDamage(CCore*, player_t*);
void P_PoisonPlayer(player_t*, mobj_t * poisoner, int poison);
void P_PoisonDamage(CCore*, player_t*, mobj_t * source, int damage, dboolean playPainSound);
dboolean P_GiveMana(CCore*, player_t*, manatype_t, int count);
dboolean Hexen_P_GiveArmor(player_t*, armortype_t, int amount);
dboolean P_MorphPlayer(CCore*, player_t*);

#endif
