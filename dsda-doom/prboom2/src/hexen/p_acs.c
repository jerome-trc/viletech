//
// Copyright(C) 1993-1996 Id Software, Inc.
// Copyright(C) 1993-2008 Raven Software
// Copyright(C) 2005-2014 Simon Howard
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

#include "doomdef.h"
#include "doomstat.h"
#include "m_misc.h"
#include "m_random.h"
#include "s_sound.h"
#include "sounds.h"
#include "w_wad.h"
#include "lprintf.h"
#include "r_main.h"
#include "p_tick.h"
#include "p_spec.h"
#include "p_inter.h"

#include "hexen/p_things.h"
#include "hexen/po_man.h"
#include "hexen/sn_sonix.h"

#include "dsda/id_list.h"
#include "dsda/map_format.h"

#include "p_acs.h"

#define MAX_SCRIPT_ARGS 3
#define SCRIPT_CONTINUE 0
#define SCRIPT_STOP 1
#define SCRIPT_TERMINATE 2
#define OPEN_SCRIPTS_BASE 1000
#define PRINT_BUFFER_SIZE 256
#define GAME_SINGLE_PLAYER 0
#define GAME_NET_COOPERATIVE 1
#define GAME_NET_DEATHMATCH 2
#define TEXTURE_TOP 0
#define TEXTURE_MIDDLE 1
#define TEXTURE_BOTTOM 2

#ifdef _MSC_VER // proff: This is the same as __attribute__ ((packed)) in GNUC
#pragma pack(push)
#pragma pack(1)
#endif //_MSC_VER

typedef struct {
    int marker;
    int infoOffset;
    int code;
} PACKEDATTR acsHeader_t;

#ifdef _MSC_VER
#pragma pack(pop)
#endif //_MSC_VER

static void StartOpenACS(int number, int infoIndex, int offset);
static void ScriptFinished(int number);
static dboolean TagBusy(int tag);
static dboolean AddToACSStore(int map, int number, byte * args);
static int GetACSIndex(int number);
static void Push(int value);
static int Pop(void);
static int Top(void);
static void Drop(void);

static int CmdNOP(CCore*);
static int CmdTerminate(CCore*);
static int CmdSuspend(CCore*);
static int CmdPushNumber(CCore*);
static int CmdLSpec1(CCore*);
static int CmdLSpec2(CCore*);
static int CmdLSpec3(CCore*);
static int CmdLSpec4(CCore*);
static int CmdLSpec5(CCore*);
static int CmdLSpec1Direct(CCore*);
static int CmdLSpec2Direct(CCore*);
static int CmdLSpec3Direct(CCore*);
static int CmdLSpec4Direct(CCore*);
static int CmdLSpec5Direct(CCore*);
static int CmdAdd(CCore*);
static int CmdSubtract(CCore*);
static int CmdMultiply(CCore*);
static int CmdDivide(CCore*);
static int CmdModulus(CCore*);
static int CmdEQ(CCore*);
static int CmdNE(CCore*);
static int CmdLT(CCore*);
static int CmdGT(CCore*);
static int CmdLE(CCore*);
static int CmdGE(CCore*);
static int CmdAssignScriptVar(CCore*);
static int CmdAssignMapVar(CCore*);
static int CmdAssignWorldVar(CCore*);
static int CmdPushScriptVar(CCore*);
static int CmdPushMapVar(CCore*);
static int CmdPushWorldVar(CCore*);
static int CmdAddScriptVar(CCore*);
static int CmdAddMapVar(CCore*);
static int CmdAddWorldVar(CCore*);
static int CmdSubScriptVar(CCore*);
static int CmdSubMapVar(CCore*);
static int CmdSubWorldVar(CCore*);
static int CmdMulScriptVar(CCore*);
static int CmdMulMapVar(CCore*);
static int CmdMulWorldVar(CCore*);
static int CmdDivScriptVar(CCore*);
static int CmdDivMapVar(CCore*);
static int CmdDivWorldVar(CCore*);
static int CmdModScriptVar(CCore*);
static int CmdModMapVar(CCore*);
static int CmdModWorldVar(CCore*);
static int CmdIncScriptVar(CCore*);
static int CmdIncMapVar(CCore*);
static int CmdIncWorldVar(CCore*);
static int CmdDecScriptVar(CCore*);
static int CmdDecMapVar(CCore*);
static int CmdDecWorldVar(CCore*);
static int CmdGoto(CCore*);
static int CmdIfGoto(CCore*);
static int CmdDrop(CCore*);
static int CmdDelay(CCore*);
static int CmdDelayDirect(CCore*);
static int CmdRandom(CCore*);
static int CmdRandomDirect(CCore*);
static int CmdThingCount(CCore*);
static int CmdThingCountDirect(CCore*);
static int CmdTagWait(CCore*);
static int CmdTagWaitDirect(CCore*);
static int CmdPolyWait(CCore*);
static int CmdPolyWaitDirect(CCore*);
static int CmdChangeFloor(CCore*);
static int CmdChangeFloorDirect(CCore*);
static int CmdChangeCeiling(CCore*);
static int CmdChangeCeilingDirect(CCore*);
static int CmdRestart(CCore*);
static int CmdAndLogical(CCore*);
static int CmdOrLogical(CCore*);
static int CmdAndBitwise(CCore*);
static int CmdOrBitwise(CCore*);
static int CmdEorBitwise(CCore*);
static int CmdNegateLogical(CCore*);
static int CmdLShift(CCore*);
static int CmdRShift(CCore*);
static int CmdUnaryMinus(CCore*);
static int CmdIfNotGoto(CCore*);
static int CmdLineSide(CCore*);
static int CmdScriptWait(CCore*);
static int CmdScriptWaitDirect(CCore*);
static int CmdClearLineSpecial(CCore*);
static int CmdCaseGoto(CCore*);
static int CmdBeginPrint(CCore*);
static int CmdEndPrint(CCore*);
static int CmdPrintString(CCore*);
static int CmdPrintNumber(CCore*);
static int CmdPrintCharacter(CCore*);
static int CmdPlayerCount(CCore*);
static int CmdGameType(CCore*);
static int CmdGameSkill(CCore*);
static int CmdTimer(CCore*);
static int CmdSectorSound(CCore*);
static int CmdAmbientSound(CCore*);
static int CmdSoundSequence(CCore*);
static int CmdSetLineTexture(CCore*);
static int CmdSetLineBlocking(CCore*);
static int CmdSetLineSpecial(CCore*);
static int CmdThingSound(CCore*);
static int CmdEndPrintBold(CCore*);

static void ThingCount(int type, int tid);

int ACScriptCount;
const byte *ActionCodeBase;
static int ActionCodeSize;
acsInfo_t *ACSInfo;
int MapVars[MAX_ACS_MAP_VARS];
int WorldVars[MAX_ACS_WORLD_VARS];
acsstore_t ACSStore[MAX_ACS_STORE + 1]; // +1 for termination marker

static char EvalContext[64];
static acs_t *ACScript;
static unsigned int PCodeOffset;
static int SpecArgs[8];
static int ACStringCount;
static const char **ACStrings;
static char PrintBuffer[PRINT_BUFFER_SIZE];
static acs_t *NewScript;

static int (*PCodeCmds[]) (CCore*) =
{
        CmdNOP,
        CmdTerminate,
        CmdSuspend,
        CmdPushNumber,
        CmdLSpec1,
        CmdLSpec2,
        CmdLSpec3,
        CmdLSpec4,
        CmdLSpec5,
        CmdLSpec1Direct,
        CmdLSpec2Direct,
        CmdLSpec3Direct,
        CmdLSpec4Direct,
        CmdLSpec5Direct,
        CmdAdd,
        CmdSubtract,
        CmdMultiply,
        CmdDivide,
        CmdModulus,
        CmdEQ,
        CmdNE,
        CmdLT,
        CmdGT,
        CmdLE,
        CmdGE,
        CmdAssignScriptVar,
        CmdAssignMapVar,
        CmdAssignWorldVar,
        CmdPushScriptVar,
        CmdPushMapVar,
        CmdPushWorldVar,
        CmdAddScriptVar,
        CmdAddMapVar,
        CmdAddWorldVar,
        CmdSubScriptVar,
        CmdSubMapVar,
        CmdSubWorldVar,
        CmdMulScriptVar,
        CmdMulMapVar,
        CmdMulWorldVar,
        CmdDivScriptVar,
        CmdDivMapVar,
        CmdDivWorldVar,
        CmdModScriptVar,
        CmdModMapVar,
        CmdModWorldVar,
        CmdIncScriptVar,
        CmdIncMapVar,
        CmdIncWorldVar,
        CmdDecScriptVar,
        CmdDecMapVar,
        CmdDecWorldVar,
        CmdGoto,
        CmdIfGoto,
        CmdDrop,
        CmdDelay,
        CmdDelayDirect,
        CmdRandom,
        CmdRandomDirect,
        CmdThingCount,
        CmdThingCountDirect,
        CmdTagWait,
        CmdTagWaitDirect,
        CmdPolyWait,
        CmdPolyWaitDirect,
        CmdChangeFloor,
        CmdChangeFloorDirect,
        CmdChangeCeiling,
        CmdChangeCeilingDirect,
        CmdRestart,
        CmdAndLogical,
        CmdOrLogical,
        CmdAndBitwise,
        CmdOrBitwise,
        CmdEorBitwise,
        CmdNegateLogical,
        CmdLShift,
        CmdRShift,
        CmdUnaryMinus,
        CmdIfNotGoto,
        CmdLineSide,
        CmdScriptWait,
        CmdScriptWaitDirect,
        CmdClearLineSpecial,
        CmdCaseGoto,
        CmdBeginPrint,
        CmdEndPrint,
        CmdPrintString,
        CmdPrintNumber,
        CmdPrintCharacter,
        CmdPlayerCount,
        CmdGameType,
        CmdGameSkill,
        CmdTimer,
        CmdSectorSound,
        CmdAmbientSound,
        CmdSoundSequence,
        CmdSetLineTexture,
        CmdSetLineBlocking,
        CmdSetLineSpecial,
        CmdThingSound,
        CmdEndPrintBold,
};

static void ACSAssert(int condition, const char *fmt, ...)
{
    char buf[128];
    va_list args;

    if (condition)
    {
        return;
    }

    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    I_Error("ACS assertion failure: in %s: %s", EvalContext, buf);
}

static int ReadCodeInt(void)
{
    int result;
    const int *ptr;

    ACSAssert(PCodeOffset + 3 < ActionCodeSize,
              "unexpectedly reached end of ACS lump");

    ptr = (const int *) (ActionCodeBase + PCodeOffset);
    result = LittleLong(*ptr);
    PCodeOffset += 4;

    return result;
}

static int ReadScriptVar(void)
{
    int var = ReadCodeInt();
    ACSAssert(var >= 0, "negative script variable: %d < 0", var);
    ACSAssert(var < MAX_ACS_SCRIPT_VARS,
              "invalid script variable: %d >= %d", var, MAX_ACS_SCRIPT_VARS);
    return var;
}

static int ReadMapVar(void)
{
    int var = ReadCodeInt();
    ACSAssert(var >= 0, "negative map variable: %d < 0", var);
    ACSAssert(var < MAX_ACS_MAP_VARS,
              "invalid map variable: %d >= %d", var, MAX_ACS_MAP_VARS);
    return var;
}

static int ReadWorldVar(void)
{
    int var = ReadCodeInt();
    ACSAssert(var >= 0, "negative world variable: %d < 0", var);
    ACSAssert(var < MAX_ACS_WORLD_VARS,
              "invalid world variable: %d >= %d", var, MAX_ACS_WORLD_VARS);
    return var;
}

static const char *StringLookup(int string_index)
{
    ACSAssert(string_index >= 0,
              "negative string index: %d < 0", string_index);
    ACSAssert(string_index < ACStringCount,
              "invalid string index: %d >= %d", string_index, ACStringCount);
    return ACStrings[string_index];
}

static int ReadOffset(void)
{
    int offset = ReadCodeInt();
    ACSAssert(offset >= 0, "negative lump offset %d", offset);
    ACSAssert(offset < ActionCodeSize, "invalid lump offset: %d >= %d",
              offset, ActionCodeSize);
    return offset;
}

void P_LoadACScripts(int lump)
{
    int i, offset;
    const acsHeader_t *header;
    acsInfo_t *info;

    ActionCodeBase = W_LumpByNum(lump);
    ActionCodeSize = W_LumpLength(lump);

    snprintf(EvalContext, sizeof(EvalContext), "header parsing of lump #%d", lump);

    header = (const acsHeader_t *) ActionCodeBase;
    PCodeOffset = LittleLong(header->infoOffset);

    ACScriptCount = ReadCodeInt();

    if (ACScriptCount == 0)
    {                           // Empty behavior lump
        return;
    }

    ACSInfo = Z_MallocLevel(ACScriptCount * sizeof(acsInfo_t));
    memset(ACSInfo, 0, ACScriptCount * sizeof(acsInfo_t));
    for (i = 0, info = ACSInfo; i < ACScriptCount; i++, info++)
    {
        info->number = ReadCodeInt();
        info->offset = ReadOffset();
        info->argCount = ReadCodeInt();

        if (info->argCount > MAX_SCRIPT_ARGS)
        {
            fprintf(stderr, "Warning: ACS script #%i has %i arguments, more "
                            "than the maximum of %i. Enforcing limit.\n"
                            "If you are seeing this message, please report "
                            "the name of the WAD where you saw it.\n",
                            i, info->argCount, MAX_SCRIPT_ARGS);
            info->argCount = MAX_SCRIPT_ARGS;
        }

        if (info->number >= OPEN_SCRIPTS_BASE)
        {                       // Auto-activate
            info->number -= OPEN_SCRIPTS_BASE;
            StartOpenACS(info->number, i, info->offset);
            info->state = ASTE_RUNNING;
        }
        else
        {
            info->state = ASTE_INACTIVE;
        }
    }

    ACStringCount = ReadCodeInt();
    ACSAssert(ACStringCount >= 0, "negative string count %d", ACStringCount);
    ACStrings = Z_MallocLevel(ACStringCount * sizeof(char *));

    for (i=0; i<ACStringCount; ++i)
    {
        offset = ReadOffset();
        ACStrings[i] = (const char *) ActionCodeBase + offset;
        ACSAssert(memchr(ACStrings[i], '\0', ActionCodeSize - offset) != NULL,
                  "string %d missing terminating NUL", i);
    }

    memset(MapVars, 0, sizeof(MapVars));
}

static void StartOpenACS(int number, int infoIndex, int offset)
{
    acs_t *script;

    script = Z_MallocLevel(sizeof(acs_t));
    memset(script, 0, sizeof(acs_t));
    script->number = number;

    // World objects are allotted 1 second for initialization
    script->delayCount = 35;

    script->infoIndex = infoIndex;
    script->ip = offset;
    script->thinker.function = T_InterpretACS;
    P_AddThinker(&script->thinker);
}

void P_CheckACSStore(CCore* cx)
{
    acsstore_t *store;

    for (store = ACSStore; store->map != 0; store++)
    {
        if (store->map == gamemap)
        {
            P_StartACS(cx, store->script, 0, store->args, NULL, NULL, 0);
            if (NewScript)
            {
                NewScript->delayCount = 35;
            }
            store->map = -1;
        }
    }
}

static char ErrorMsg[128];

dboolean P_StartACS(CCore* cx, int number, int map, byte * args, mobj_t * activator,
                   line_t * line, int side)
{
    int i;
    acs_t *script;
    int infoIndex;
    aste_t *statePtr;

    NewScript = NULL;
    if (map && map != gamemap)
    {                           // Add to the script store
        return AddToACSStore(map, number, args);
    }
    infoIndex = GetACSIndex(number);
    if (infoIndex == -1)
    {                           // Script not found
        //I_Error("P_StartACS: Unknown script number %d", number);
        snprintf(ErrorMsg, sizeof(ErrorMsg), "P_STARTACS ERROR: UNKNOWN SCRIPT %d", number);
        P_SetMessage(cx, &players[consoleplayer], ErrorMsg, true);
    }
    statePtr = &ACSInfo[infoIndex].state;
    if (*statePtr == ASTE_SUSPENDED)
    {                           // Resume a suspended script
        *statePtr = ASTE_RUNNING;
        return true;
    }
    if (*statePtr != ASTE_INACTIVE)
    {                           // Script is already executing
        return false;
    }
    script = Z_MallocLevel(sizeof(acs_t));
    memset(script, 0, sizeof(acs_t));
    script->number = number;
    script->infoIndex = infoIndex;
    P_SetTarget(&script->activator, activator);
    script->line = line;
    script->side = side;
    script->ip = ACSInfo[infoIndex].offset;
    script->thinker.function = T_InterpretACS;
    for (i = 0; i < MAX_SCRIPT_ARGS && i < ACSInfo[infoIndex].argCount; i++)
    {
        script->vars[i] = args[i];
    }
    *statePtr = ASTE_RUNNING;
    P_AddThinker(&script->thinker);
    NewScript = script;
    return true;
}

static dboolean AddToACSStore(int map, int number, byte * args)
{
    int i;
    int index;

    index = -1;
    for (i = 0; ACSStore[i].map != 0; i++)
    {
        if (ACSStore[i].script == number && ACSStore[i].map == map)
        {                       // Don't allow duplicates
            return false;
        }
        if (index == -1 && ACSStore[i].map == -1)
        {                       // Remember first empty slot
            index = i;
        }
    }
    if (index == -1)
    {                           // Append required
        if (i == MAX_ACS_STORE)
        {
            I_Error("AddToACSStore: MAX_ACS_STORE (%d) exceeded.",
                    MAX_ACS_STORE);
        }
        index = i;
        ACSStore[index + 1].map = 0;
    }
    ACSStore[index].map = map;
    ACSStore[index].script = number;
    memcpy(ACSStore[index].args, args, MAX_SCRIPT_ARGS);
    return true;
}

dboolean P_StartLockedACS(CCore* cx, line_t * line, byte * args, mobj_t * mo, int side)
{
    int i;
    int lock;
    byte newArgs[5];

    extern char *TextKeyMessages[11];
    extern char LockedBuffer[80];

    lock = args[4];

    if (!mo->player) {
        return false;
    }

    if (lock) {
        if (!mo->player->cards[lock - 1]) {
            snprintf(LockedBuffer, sizeof(LockedBuffer),
                     "YOU NEED THE %s\n", TextKeyMessages[lock - 1]);
            P_SetMessage(cx, mo->player, LockedBuffer, true);
            S_StartMobjSound(mo, hexen_sfx_door_locked);
            return false;
        }
    }

    for (i = 0; i < 4; i++) {
        newArgs[i] = args[i];
    }

    newArgs[4] = 0;

    return P_StartACS(cx, newArgs[0], newArgs[1], &newArgs[2], mo, line, side);
}

dboolean P_TerminateACS(int number, int map)
{
    int infoIndex;

    infoIndex = GetACSIndex(number);
    if (infoIndex == -1)
    {                           // Script not found
        return false;
    }
    if (ACSInfo[infoIndex].state == ASTE_INACTIVE
        || ACSInfo[infoIndex].state == ASTE_TERMINATING)
    {                           // States that disallow termination
        return false;
    }
    ACSInfo[infoIndex].state = ASTE_TERMINATING;
    return true;
}

dboolean P_SuspendACS(int number, int map)
{
    int infoIndex;

    infoIndex = GetACSIndex(number);
    if (infoIndex == -1)
    {                           // Script not found
        return false;
    }
    if (ACSInfo[infoIndex].state == ASTE_INACTIVE
        || ACSInfo[infoIndex].state == ASTE_SUSPENDED
        || ACSInfo[infoIndex].state == ASTE_TERMINATING)
    {                           // States that disallow suspension
        return false;
    }
    ACSInfo[infoIndex].state = ASTE_SUSPENDED;
    return true;
}

void P_ACSInitNewGame(void)
{
    memset(WorldVars, 0, sizeof(WorldVars));
    memset(ACSStore, 0, sizeof(ACSStore));
}

void T_InterpretACS(CCore* cx, void* v)
{
    acs_t* script = v;
    int cmd;
    int action;

    if (ACSInfo[script->infoIndex].state == ASTE_TERMINATING)
    {
        ACSInfo[script->infoIndex].state = ASTE_INACTIVE;
        ScriptFinished(ACScript->number);
        P_RemoveThinker(&ACScript->thinker);
        return;
    }
    if (ACSInfo[script->infoIndex].state != ASTE_RUNNING)
    {
        return;
    }
    if (script->delayCount)
    {
        script->delayCount--;
        return;
    }
    ACScript = script;
    PCodeOffset = ACScript->ip;

    do
    {
        snprintf(EvalContext, sizeof(EvalContext), "script %d @0x%x",
                 ACSInfo[script->infoIndex].number, PCodeOffset);
        cmd = ReadCodeInt();
        snprintf(EvalContext, sizeof(EvalContext), "script %d @0x%x, cmd=%d",
                 ACSInfo[script->infoIndex].number, PCodeOffset, cmd);
        ACSAssert(cmd >= 0, "negative ACS instruction %d", cmd);
        ACSAssert(cmd < arrlen(PCodeCmds),
                  "invalid ACS instruction %d (maybe this WAD is designed "
                  "for an advanced source port and is not vanilla "
                  "compatible)", cmd);
        action = PCodeCmds[cmd](cx);
    } while (action == SCRIPT_CONTINUE);

    ACScript->ip = PCodeOffset;

    if (action == SCRIPT_TERMINATE)
    {
        ACSInfo[script->infoIndex].state = ASTE_INACTIVE;
        ScriptFinished(ACScript->number);
        P_RemoveThinker(&ACScript->thinker);
    }
}

void P_TagFinished(int tag)
{
    int i;

    if (!map_format.acs) return;

    if (TagBusy(tag) == true)
    {
        return;
    }
    for (i = 0; i < ACScriptCount; i++)
    {
        if (ACSInfo[i].state == ASTE_WAITINGFORTAG
            && ACSInfo[i].waitValue == tag)
        {
            ACSInfo[i].state = ASTE_RUNNING;
        }
    }
}

void P_PolyobjFinished(int po)
{
    int i;

    if (PO_Busy(po) == true)
    {
        return;
    }
    for (i = 0; i < ACScriptCount; i++)
    {
        if (ACSInfo[i].state == ASTE_WAITINGFORPOLY
            && ACSInfo[i].waitValue == po)
        {
            ACSInfo[i].state = ASTE_RUNNING;
        }
    }
}

static void ScriptFinished(int number)
{
    int i;

    for (i = 0; i < ACScriptCount; i++)
    {
        if (ACSInfo[i].state == ASTE_WAITINGFORSCRIPT
            && ACSInfo[i].waitValue == number)
        {
            ACSInfo[i].state = ASTE_RUNNING;
        }
    }
}

static dboolean TagBusy(int tag)
{
    const int *id_p;

    FIND_SECTORS(id_p, tag)
    {
        if (sectors[*id_p].floordata || sectors[*id_p].ceilingdata)
        {
            return true;
        }
    }
    return false;
}

static int GetACSIndex(int number)
{
    int i;

    for (i = 0; i < ACScriptCount; i++)
    {
        if (ACSInfo[i].number == number)
        {
            return i;
        }
    }
    return -1;
}

void CheckACSPresent(int number)
{
    if (GetACSIndex(number) == -1)
    {
        I_Error("Required ACS script %d not initialized", number);
    }
}

static void Push(int value)
{
    ACSAssert(ACScript->stackPtr < ACS_STACK_DEPTH,
              "maximum stack depth exceeded: %d >= %d",
              ACScript->stackPtr, ACS_STACK_DEPTH);
    ACScript->stack[ACScript->stackPtr++] = value;
}

static int Pop(void)
{
    ACSAssert(ACScript->stackPtr > 0, "pop of empty stack");
    return ACScript->stack[--ACScript->stackPtr];
}

static int Top(void)
{
    ACSAssert(ACScript->stackPtr > 0, "read from top of empty stack");
    return ACScript->stack[ACScript->stackPtr - 1];
}

static void Drop(void)
{
    ACSAssert(ACScript->stackPtr > 0, "drop on empty stack");
    ACScript->stackPtr--;
}

static int CmdNOP(CCore* cx)
{
    (void)cx;
    return SCRIPT_CONTINUE;
}

static int CmdTerminate(CCore* cx)
{
    (void)cx;
    return SCRIPT_TERMINATE;
}

static int CmdSuspend(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].state = ASTE_SUSPENDED;
    return SCRIPT_STOP;
}

static int CmdPushNumber(CCore* cx)
{
    Push(ReadCodeInt());
    return SCRIPT_CONTINUE;
}

static int CmdLSpec1(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = Pop();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec2(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[1] = Pop();
    SpecArgs[0] = Pop();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec3(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[2] = Pop();
    SpecArgs[1] = Pop();
    SpecArgs[0] = Pop();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec4(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[3] = Pop();
    SpecArgs[2] = Pop();
    SpecArgs[1] = Pop();
    SpecArgs[0] = Pop();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec5(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[4] = Pop();
    SpecArgs[3] = Pop();
    SpecArgs[2] = Pop();
    SpecArgs[1] = Pop();
    SpecArgs[0] = Pop();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec1Direct(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = ReadCodeInt();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec2Direct(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = ReadCodeInt();
    SpecArgs[1] = ReadCodeInt();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec3Direct(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = ReadCodeInt();
    SpecArgs[1] = ReadCodeInt();
    SpecArgs[2] = ReadCodeInt();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec4Direct(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = ReadCodeInt();
    SpecArgs[1] = ReadCodeInt();
    SpecArgs[2] = ReadCodeInt();
    SpecArgs[3] = ReadCodeInt();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdLSpec5Direct(CCore* cx)
{
    int special;

    special = ReadCodeInt();
    SpecArgs[0] = ReadCodeInt();
    SpecArgs[1] = ReadCodeInt();
    SpecArgs[2] = ReadCodeInt();
    SpecArgs[3] = ReadCodeInt();
    SpecArgs[4] = ReadCodeInt();
    map_format.execute_line_special(cx, special, SpecArgs, ACScript->line,
                                    ACScript->side, ACScript->activator);
    return SCRIPT_CONTINUE;
}

static int CmdAdd(CCore* cx)
{
    Push(Pop() + Pop());
    return SCRIPT_CONTINUE;
}

static int CmdSubtract(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() - operand2);
    return SCRIPT_CONTINUE;
}

static int CmdMultiply(CCore* cx)
{
    Push(Pop() * Pop());
    return SCRIPT_CONTINUE;
}

static int CmdDivide(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() / operand2);
    return SCRIPT_CONTINUE;
}

static int CmdModulus(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() % operand2);
    return SCRIPT_CONTINUE;
}

static int CmdEQ(CCore* cx)
{
    Push(Pop() == Pop());
    return SCRIPT_CONTINUE;
}

static int CmdNE(CCore* cx)
{
    Push(Pop() != Pop());
    return SCRIPT_CONTINUE;
}

static int CmdLT(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() < operand2);
    return SCRIPT_CONTINUE;
}

static int CmdGT(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() > operand2);
    return SCRIPT_CONTINUE;
}

static int CmdLE(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() <= operand2);
    return SCRIPT_CONTINUE;
}

static int CmdGE(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() >= operand2);
    return SCRIPT_CONTINUE;
}

static int CmdAssignScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] = Pop();
    return SCRIPT_CONTINUE;
}

static int CmdAssignMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] = Pop();
    return SCRIPT_CONTINUE;
}

static int CmdAssignWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] = Pop();
    return SCRIPT_CONTINUE;
}

static int CmdPushScriptVar(CCore* cx)
{
    Push(ACScript->vars[ReadScriptVar()]);
    return SCRIPT_CONTINUE;
}

static int CmdPushMapVar(CCore* cx)
{
    Push(MapVars[ReadMapVar()]);
    return SCRIPT_CONTINUE;
}

static int CmdPushWorldVar(CCore* cx)
{
    Push(WorldVars[ReadWorldVar()]);
    return SCRIPT_CONTINUE;
}

static int CmdAddScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] += Pop();
    return SCRIPT_CONTINUE;
}

static int CmdAddMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] += Pop();
    return SCRIPT_CONTINUE;
}

static int CmdAddWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] += Pop();
    return SCRIPT_CONTINUE;
}

static int CmdSubScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] -= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdSubMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] -= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdSubWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] -= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdMulScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] *= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdMulMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] *= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdMulWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] *= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdDivScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] /= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdDivMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] /= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdDivWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] /= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdModScriptVar(CCore* cx)
{
    ACScript->vars[ReadScriptVar()] %= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdModMapVar(CCore* cx)
{
    MapVars[ReadMapVar()] %= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdModWorldVar(CCore* cx)
{
    WorldVars[ReadWorldVar()] %= Pop();
    return SCRIPT_CONTINUE;
}

static int CmdIncScriptVar(CCore* cx)
{
    ++ACScript->vars[ReadScriptVar()];
    return SCRIPT_CONTINUE;
}

static int CmdIncMapVar(CCore* cx)
{
    ++MapVars[ReadMapVar()];
    return SCRIPT_CONTINUE;
}

static int CmdIncWorldVar(CCore* cx)
{
    ++WorldVars[ReadWorldVar()];
    return SCRIPT_CONTINUE;
}

static int CmdDecScriptVar(CCore* cx)
{
    --ACScript->vars[ReadScriptVar()];
    return SCRIPT_CONTINUE;
}

static int CmdDecMapVar(CCore* cx)
{
    --MapVars[ReadMapVar()];
    return SCRIPT_CONTINUE;
}

static int CmdDecWorldVar(CCore* cx)
{
    --WorldVars[ReadWorldVar()];
    return SCRIPT_CONTINUE;
}

static int CmdGoto(CCore* cx)
{
    PCodeOffset = ReadOffset();
    return SCRIPT_CONTINUE;
}

static int CmdIfGoto(CCore* cx)
{
    int offset;

    offset = ReadOffset();

    if (Pop() != 0)
    {
        PCodeOffset = offset;
    }
    return SCRIPT_CONTINUE;
}

static int CmdDrop(CCore* cx)
{
    Drop();
    return SCRIPT_CONTINUE;
}

static int CmdDelay(CCore* cx)
{
    ACScript->delayCount = Pop();
    return SCRIPT_STOP;
}

static int CmdDelayDirect(CCore* cx)
{
    ACScript->delayCount = ReadCodeInt();
    return SCRIPT_STOP;
}

static int CmdRandom(CCore* cx)
{
    int low;
    int high;

    high = Pop();
    low = Pop();
    Push(low + (P_Random(pr_hexen) % (high - low + 1)));
    return SCRIPT_CONTINUE;
}

static int CmdRandomDirect(CCore* cx)
{
    int low;
    int high;

    low = ReadCodeInt();
    high = ReadCodeInt();
    Push(low + (P_Random(pr_hexen) % (high - low + 1)));
    return SCRIPT_CONTINUE;
}

static int CmdThingCount(CCore* cx)
{
    int tid;

    tid = Pop();
    ThingCount(Pop(), tid);
    return SCRIPT_CONTINUE;
}

static int CmdThingCountDirect(CCore* cx)
{
    int type;

    type = ReadCodeInt();
    ThingCount(type, ReadCodeInt());
    return SCRIPT_CONTINUE;
}

static void ThingCount(int type, int tid)
{
    int count;
    int searcher;
    mobj_t *mobj;
    mobjtype_t moType;
    thinker_t *think;

    if (!(type + tid))
    {                           // Nothing to count
        return;
    }
    moType = TranslateThingType[type];
    count = 0;
    searcher = -1;
    if (tid)
    {                           // Count TID things
        while ((mobj = P_FindMobjFromTID(tid, &searcher)) != NULL)
        {
            if (type == 0)
            {                   // Just count TIDs
                count++;
            }
            else if (moType == mobj->type)
            {
                if (mobj->flags & MF_COUNTKILL && mobj->health <= 0)
                {               // Don't count dead monsters
                    continue;
                }
                count++;
            }
        }
    }
    else
    {                           // Count only types
        for (think = thinkercap.next; think != &thinkercap;
             think = think->next)
        {
            if (think->function != P_MobjThinker)
            {                   // Not a mobj thinker
                continue;
            }
            mobj = (mobj_t *) think;
            if (mobj->type != moType)
            {                   // Doesn't match
                continue;
            }
            if (mobj->flags & MF_COUNTKILL && mobj->health <= 0)
            {                   // Don't count dead monsters
                continue;
            }
            count++;
        }
    }
    Push(count);
}

static int CmdTagWait(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = Pop();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORTAG;
    return SCRIPT_STOP;
}

static int CmdTagWaitDirect(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = ReadCodeInt();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORTAG;
    return SCRIPT_STOP;
}

static int CmdPolyWait(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = Pop();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORPOLY;
    return SCRIPT_STOP;
}

static int CmdPolyWaitDirect(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = ReadCodeInt();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORPOLY;
    return SCRIPT_STOP;
}

static int CmdChangeFloor(CCore* cx)
{
    int tag;
    int flat;
    const int *id_p;

    flat = R_FlatNumForName(StringLookup(Pop()));
    tag = Pop();
    FIND_SECTORS(id_p, tag)
    {
        sectors[*id_p].floorpic = flat;
    }
    return SCRIPT_CONTINUE;
}

static int CmdChangeFloorDirect(CCore* cx)
{
    int tag;
    int flat;
    const int *id_p;

    tag = ReadCodeInt();
    flat = R_FlatNumForName(StringLookup(ReadCodeInt()));
    FIND_SECTORS(id_p, tag)
    {
        sectors[*id_p].floorpic = flat;
    }
    return SCRIPT_CONTINUE;
}

static int CmdChangeCeiling(CCore* cx)
{
    int tag;
    int flat;
    const int *id_p;

    flat = R_FlatNumForName(StringLookup(Pop()));
    tag = Pop();
    FIND_SECTORS(id_p, tag)
    {
        sectors[*id_p].ceilingpic = flat;
    }
    return SCRIPT_CONTINUE;
}

static int CmdChangeCeilingDirect(CCore* cx)
{
    int tag;
    int flat;
    const int *id_p;

    tag = ReadCodeInt();
    flat = R_FlatNumForName(StringLookup(ReadCodeInt()));
    FIND_SECTORS(id_p, tag)
    {
        sectors[*id_p].ceilingpic = flat;
    }
    return SCRIPT_CONTINUE;
}

static int CmdRestart(CCore* cx)
{
    PCodeOffset = ACSInfo[ACScript->infoIndex].offset;
    return SCRIPT_CONTINUE;
}

static int CmdAndLogical(CCore* cx)
{
    Push(Pop() && Pop());
    return SCRIPT_CONTINUE;
}

static int CmdOrLogical(CCore* cx)
{
    Push(Pop() || Pop());
    return SCRIPT_CONTINUE;
}

static int CmdAndBitwise(CCore* cx)
{
    Push(Pop() & Pop());
    return SCRIPT_CONTINUE;
}

static int CmdOrBitwise(CCore* cx)
{
    Push(Pop() | Pop());
    return SCRIPT_CONTINUE;
}

static int CmdEorBitwise(CCore* cx)
{
    Push(Pop() ^ Pop());
    return SCRIPT_CONTINUE;
}

static int CmdNegateLogical(CCore* cx)
{
    Push(!Pop());
    return SCRIPT_CONTINUE;
}

static int CmdLShift(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() << operand2);
    return SCRIPT_CONTINUE;
}

static int CmdRShift(CCore* cx)
{
    int operand2;

    operand2 = Pop();
    Push(Pop() >> operand2);
    return SCRIPT_CONTINUE;
}

static int CmdUnaryMinus(CCore* cx)
{
    Push(-Pop());
    return SCRIPT_CONTINUE;
}

static int CmdIfNotGoto(CCore* cx)
{
    int offset;

    offset = ReadOffset();

    if (Pop() == 0)
    {
        PCodeOffset = offset;
    }
    return SCRIPT_CONTINUE;
}

static int CmdLineSide(CCore* cx)
{
    Push(ACScript->side);
    return SCRIPT_CONTINUE;
}

static int CmdScriptWait(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = Pop();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORSCRIPT;
    return SCRIPT_STOP;
}

static int CmdScriptWaitDirect(CCore* cx)
{
    ACSInfo[ACScript->infoIndex].waitValue = ReadCodeInt();
    ACSInfo[ACScript->infoIndex].state = ASTE_WAITINGFORSCRIPT;
    return SCRIPT_STOP;
}

static int CmdClearLineSpecial(CCore* cx)
{
    if (ACScript->line)
    {
        ACScript->line->special = 0;
    }
    return SCRIPT_CONTINUE;
}

static int CmdCaseGoto(CCore* cx)
{
    int value;
    int offset;

    value = ReadCodeInt();
    offset = ReadOffset();

    if (Top() == value)
    {
        PCodeOffset = offset;
        Drop();
    }

    return SCRIPT_CONTINUE;
}

static int CmdBeginPrint(CCore* cx)
{
    *PrintBuffer = 0;
    return SCRIPT_CONTINUE;
}

static int CmdEndPrint(CCore* cx)
{
    player_t *player;

    if (ACScript->activator && ACScript->activator->player)
    {
        player = ACScript->activator->player;
    }
    else
    {
        player = &players[consoleplayer];
    }
    P_SetMessage(cx, player, PrintBuffer, true);
    return SCRIPT_CONTINUE;
}

static int CmdEndPrintBold(CCore* cx)
{
    int i;

    for (i = 0; i < g_maxplayers; i++)
    {
        if (playeringame[i])
        {
            P_SetYellowMessage(cx, &players[i], PrintBuffer, true);
        }
    }
    return SCRIPT_CONTINUE;
}

static int CmdPrintString(CCore* cx)
{
    M_StringConcat(PrintBuffer, StringLookup(Pop()), sizeof(PrintBuffer));
    return SCRIPT_CONTINUE;
}

static int CmdPrintNumber(CCore* cx)
{
    char tempStr[16];

    snprintf(tempStr, sizeof(tempStr), "%d", Pop());
    M_StringConcat(PrintBuffer, tempStr, sizeof(PrintBuffer));
    return SCRIPT_CONTINUE;
}

static int CmdPrintCharacter(CCore* cx)
{
    char tempStr[2];

    tempStr[0] = Pop();
    tempStr[1] = '\0';
    M_StringConcat(PrintBuffer, tempStr, sizeof(PrintBuffer));

    return SCRIPT_CONTINUE;
}

static int CmdPlayerCount(CCore* cx)
{
    int i;
    int count;

    count = 0;
    for (i = 0; i < g_maxplayers; i++)
    {
        count += playeringame[i];
    }
    Push(count);
    return SCRIPT_CONTINUE;
}

static int CmdGameType(CCore* cx)
{
    int gametype;

    if (netgame == false)
    {
        gametype = GAME_SINGLE_PLAYER;
    }
    else if (deathmatch)
    {
        gametype = GAME_NET_DEATHMATCH;
    }
    else
    {
        gametype = GAME_NET_COOPERATIVE;
    }
    Push(gametype);
    return SCRIPT_CONTINUE;
}

static int CmdGameSkill(CCore* cx)
{
    Push(gameskill);
    return SCRIPT_CONTINUE;
}

static int CmdTimer(CCore* cx)
{
    Push(leveltime);
    return SCRIPT_CONTINUE;
}

static int CmdSectorSound(CCore* cx)
{
    int volume;
    mobj_t *mobj;

    mobj = NULL;
    if (ACScript->line)
    {
        mobj = (mobj_t *) & ACScript->line->frontsector->soundorg;
    }
    volume = Pop();
    S_StartSoundAtVolume(mobj, S_GetSoundID(StringLookup(Pop())), volume, 0);
    return SCRIPT_CONTINUE;
}

static int CmdThingSound(CCore* cx)
{
    int tid;
    int sound;
    int volume;
    mobj_t *mobj;
    int searcher;

    volume = Pop();
    sound = S_GetSoundID(StringLookup(Pop()));
    tid = Pop();
    searcher = -1;
    while ((mobj = P_FindMobjFromTID(tid, &searcher)) != NULL)
    {
        S_StartSoundAtVolume(mobj, sound, volume, 0);
    }
    return SCRIPT_CONTINUE;
}

static int CmdAmbientSound(CCore* cx)
{
    int volume;

    volume = Pop();
    S_StartSoundAtVolume(NULL, S_GetSoundID(StringLookup(Pop())), volume, 0);
    return SCRIPT_CONTINUE;
}

static int CmdSoundSequence(CCore* cx)
{
    mobj_t *mobj;

    mobj = NULL;
    if (ACScript->line)
    {
        mobj = (mobj_t *) & ACScript->line->frontsector->soundorg;
    }
    SN_StartSequenceName(mobj, StringLookup(Pop()));
    return SCRIPT_CONTINUE;
}

static int CmdSetLineTexture(CCore* cx)
{
    line_t *line;
    int lineTag;
    int side;
    int position;
    int texture;
    int searcher;

    texture = R_TextureNumForName(StringLookup(Pop()));
    position = Pop();
    side = Pop();
    lineTag = Pop();
    searcher = -1;
    while ((line = P_FindLine(lineTag, &searcher)) != NULL)
    {
        if (position == TEXTURE_MIDDLE)
        {
            sides[line->sidenum[side]].midtexture = texture;
        }
        else if (position == TEXTURE_BOTTOM)
        {
            sides[line->sidenum[side]].bottomtexture = texture;
        }
        else
        {                       // TEXTURE_TOP
            sides[line->sidenum[side]].toptexture = texture;
        }
    }
    return SCRIPT_CONTINUE;
}

static int CmdSetLineBlocking(CCore* cx)
{
    line_t *line;
    int lineTag;
    dboolean blocking;
    int searcher;

    blocking = Pop()? ML_BLOCKING : 0;
    lineTag = Pop();
    searcher = -1;
    while ((line = P_FindLine(lineTag, &searcher)) != NULL)
    {
        line->flags = (line->flags & ~ML_BLOCKING) | blocking;
    }
    return SCRIPT_CONTINUE;
}

static int CmdSetLineSpecial(CCore* cx)
{
    line_t *line;
    int lineTag;
    int special, arg1, arg2, arg3, arg4, arg5;
    int searcher;

    arg5 = Pop();
    arg4 = Pop();
    arg3 = Pop();
    arg2 = Pop();
    arg1 = Pop();
    special = Pop();
    lineTag = Pop();
    searcher = -1;
    while ((line = P_FindLine(lineTag, &searcher)) != NULL)
    {
        line->special = special;
        line->special_args[0] = arg1;
        line->special_args[1] = arg2;
        line->special_args[2] = arg3;
        line->special_args[3] = arg4;
        line->special_args[4] = arg5;
    }
    return SCRIPT_CONTINUE;
}
