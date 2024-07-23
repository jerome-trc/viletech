//
// Copyright(C) 2021 by Ryan Krafnick
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
//	DSDA Extended HUD
//

#ifndef __DSDA_EXHUD__
#define __DSDA_EXHUD__

#include "viletech.zig.h"

void dsda_InitExHud(CCore*);
void dsda_UpdateExHud(void);
void dsda_DrawExHud(CCore*);
void dsda_DrawExIntermission(CCore*);
void dsda_ToggleRenderStats(void);
void dsda_RefreshExHudFPS(CCore*);
void dsda_RefreshExHudMinimap(CCore*);
void dsda_RefreshExHudLevelSplits(CCore*);
void dsda_RefreshExHudCoordinateDisplay(CCore*);
void dsda_RefreshExHudCommandDisplay(CCore*);
void dsda_RefreshMapCoordinates(CCore*);
void dsda_RefreshMapTotals(CCore*);
void dsda_RefreshMapTime(CCore*);
void dsda_RefreshMapTitle(CCore*);

#endif
