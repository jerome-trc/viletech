#pragma once

#include <SDL_events.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include <SDL.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

typedef int32_t GameTick;

struct player_s;
struct pspdef_s;
typedef struct Core Core;

typedef struct CCore {
    Core* core;
    bool devgui_open;
    void* imgui_ctx;
    GameTick saved_gametick;
} CCore;

void coreDeinit(CCore*);

// DeHackEd action pointers ////////////////////////////////////////////////////

void A_BurstShotgunFire(CCore*, struct player_s*, struct pspdef_s*);

void A_BurstShotgunCheckVent(CCore*, struct player_s*, struct pspdef_s*);

// Developer GUI ///////////////////////////////////////////////////////////////

void addConsoleToast(CCore*, const char*);

void dguiLayout(CCore*);

void dguiFrameBegin(CCore*);

void dguiFrameFinish(CCore*);

void dguiFrameDraw(CCore*);

void dguiSetup(CCore*, SDL_Window*, void* sdl_gl_ctx);

void dguiShutdown(void);

bool dguiWantsKeyboard(CCore*);

bool dguiWantsMouse(CCore*);

bool dguiProcessEvent(CCore*, SDL_Event*);

void populateMusicPlayer(CCore*);

// Platform ////////////////////////////////////////////////////////////////////

/// Retrieve embedded window icon data.
const uint8_t* windowIcon(int32_t* size);

// Plugin //////////////////////////////////////////////////////////////////////

void addPlugin(CCore* cx, const char* path);

void loadPlugins(CCore*);

#else // if !defined(RATBOOM_ZIG)

typedef struct CCore CCore;

#endif
