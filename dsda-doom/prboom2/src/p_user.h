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
 *      Player related stuff.
 *      Bobbing POV/weapon, movement.
 *      Pending weapon.
 *
 *-----------------------------------------------------------------------------*/

#ifndef __P_USER__
#define __P_USER__

#include "d_player.h"

void P_PlayerThink(CCore*, player_t*);
void P_CalcHeight(player_t*);
void P_DeathThink(CCore*, player_t*);
void P_MovePlayer(CCore*, player_t*);
void P_ForwardThrust(player_t*, angle_t angle, fixed_t move);
void P_Thrust(player_t*, angle_t angle, fixed_t move);

void P_SetPitch(player_t*);

// heretic

int P_GetPlayerNum(player_t*);
void P_PlayerRemoveArtifact(player_t*, int slot);
void P_PlayerUseArtifact(CCore*, player_t*, artitype_t);
void P_PlayerNextArtifact(player_t*);
dboolean P_UseArtifact(CCore*, player_t*, artitype_t);
void P_ChickenPlayerThink(CCore*, player_t*);
dboolean P_UndoPlayerChicken(CCore*, player_t*);
void Raven_P_MovePlayer(CCore*, player_t*);

// hexen

void ResetBlasted(mobj_t*);
void P_TeleportOther(CCore*, mobj_t*);
dboolean P_UndoPlayerMorph(CCore*, player_t*);
void P_MorphPlayerThink(player_t*);

void P_PlayerEndFlight(player_t*);

#endif  /* __P_USER__ */
