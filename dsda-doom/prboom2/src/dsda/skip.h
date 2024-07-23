//
// Copyright(C) 2022 by Ryan Krafnick
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
//	DSDA Skip Mode
//

#include "doomtype.h"

#include "viletech.zig.h"

dboolean dsda_SkipMode(void);
void dsda_EnterSkipMode(CCore*);
void dsda_ExitSkipMode(CCore*);
void dsda_ToggleSkipMode(CCore*);
void dsda_SkipToNextMap(CCore*);
void dsda_SkipToEndOfMap(CCore*);
void dsda_SkipToLogicTic(CCore*, int tic);
void dsda_EvaluateSkipModeGTicker(CCore*);
void dsda_EvaluateSkipModeInitNew(void);
void dsda_EvaluateSkipModeBuildTiccmd(CCore*);
void dsda_EvaluateSkipModeDoCompleted(CCore*);
void dsda_EvaluateSkipModeDoTeleportNewMap(CCore*);
void dsda_EvaluateSkipModeDoWorldDone(CCore*);
void dsda_EvaluateSkipModeCheckDemoStatus(CCore*);
void dsda_HandleSkip(CCore*);
