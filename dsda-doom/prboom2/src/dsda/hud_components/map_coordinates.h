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
//	DSDA Map Coordinates HUD Component
//

#ifndef __DSDA_HUD_COMPONENT_MAP_COORDINATES__
#define __DSDA_HUD_COMPONENT_MAP_COORDINATES__

struct CCore;

void dsda_InitMapCoordinatesHC(int x_offset, int y_offset, int vpt_flags, int* args, int arg_count, void** data);
void dsda_UpdateMapCoordinatesHC(void* data);
void dsda_DrawMapCoordinatesHC(struct CCore*, void* data);

#endif