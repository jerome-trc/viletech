#pragma once

#include <SDL_events.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include <SDL.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

typedef int32_t GameTick;

typedef struct Core Core;

typedef struct CCore {
    Core* core;
    bool devgui_open;
    void* imgui_ctx;
    GameTick saved_gametick;
} CCore;

static inline void vt_addDynLib(CCore* cx, char* path) {}

static inline void vt_addConsoleToast(CCore* self, char* msg) {}

void dguiLayout(CCore*);

void dguiFrameBegin(CCore*);

void dguiFrameFinish(CCore*);

void dguiFrameDraw(CCore*);

void dguiSetup(CCore*, SDL_Window*, void* sdl_gl_ctx);

void dguiShutdown(void);

bool dguiWantsKeyboard(CCore*);

bool dguiWantsMouse(CCore*);

static inline void vt_loadDynLibs(CCore* cx) {}

bool dguiProcessEvent(CCore*, SDL_Event*);

static inline void vt_writeEngineTime(void) {}

/// Retrieve embedded window icon data.
const uint8_t* windowIcon(int32_t* size);

#else // if !defined(RATBOOM_ZIG)

typedef struct CCore CCore;

#endif
