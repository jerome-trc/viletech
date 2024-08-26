#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include "viletech/posix.h"

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

typedef int32_t GameTick;

struct player_s;
struct pspdef_s;
typedef struct Core Core;

typedef struct CCore {
    bool devgui_open;
    void* imgui_ctx;
    GameTick saved_gametick;
} CCore;

void coreDeinit(CCore*);

void registerPref(CCore*, const char* pref_v);

// Game ////////////////////////////////////////////////////////////////////////

void loadLevel(CCore*);

// Platform ////////////////////////////////////////////////////////////////////

/// Retrieve embedded window icon data.
const uint8_t* windowIcon(int32_t* size);

// Plugin //////////////////////////////////////////////////////////////////////

void addPlugin(CCore* cx, const char* path);

void loadPlugins(CCore*);

#else // if !defined(RATBOOM_ZIG)

typedef struct CCore CCore;

#endif // if !defined(RATBOOM_ZIG)
