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
//	DSDA Build Mode
//

#include "d_event.h"
#include "d_ticcmd.h"

#include "viletech.zig.h"

dboolean dsda_AllowBuilding(void);
dboolean dsda_BuildMode(void);
void dsda_QueueBuildCommands(ticcmd_t* cmds, int depth);
dboolean dsda_BuildPlayback(void);
void dsda_CopyBuildCmd(ticcmd_t* cmd);
void dsda_ReadBuildCmd(CCore*, ticcmd_t* cmd);
void dsda_EnterBuildMode(CCore*);
void dsda_RefreshBuildMode(void);
dboolean dsda_BuildResponder(CCore*, event_t*);
void dsda_ToggleBuildTurbo(void);
dboolean dsda_AdvanceFrame(void);
dboolean dsda_BuildMF(CCore*, int x);
dboolean dsda_BuildMB(CCore*, int x);
dboolean dsda_BuildSR(CCore*, int x);
dboolean dsda_BuildSL(CCore*, int x);
dboolean dsda_BuildTR(CCore*, int x);
dboolean dsda_BuildTL(CCore*, int x);
dboolean dsda_BuildFU(CCore*,  int x);
dboolean dsda_BuildFD(CCore*, int x);
dboolean dsda_BuildFC(CCore*);
dboolean dsda_BuildLU(CCore*, int x);
dboolean dsda_BuildLD(CCore*, int x);
dboolean dsda_BuildLC(CCore*);
dboolean dsda_BuildUA(CCore*, int x);
