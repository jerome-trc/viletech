//
// Copyright(C) 2023 by Ryan Krafnick
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
// DESCRIPTION:
//	DSDA Message
//

#ifndef __DSDA_MESSAGE__
#define __DSDA_MESSAGE__

#include "d_player.h"

void dsda_AddPlayerAlert(CCore*, const char* str, player_t*);
void dsda_AddAlert(CCore*, const char* str);
void dsda_AddPlayerMessage(CCore*, const char* str, player_t*);
void dsda_AddMessage(CCore*, const char* str);
void dsda_AddUnblockableMessage(CCore*, const char* str);
void dsda_UpdateMessenger(void);
void dsda_InitMessenger(void);
void dsda_ReplayMessage(CCore*);

#endif
