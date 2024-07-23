#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

typedef struct Core Core;
typedef void* LibHandle;
typedef union SdlEvent SdlEvent;
typedef struct SdlWindow SdlWindow;

typedef int32_t GameTick;

typedef struct CCore {
    Core* core;
    GameTick saved_gametick;
} CCore;

static inline void vt_addDynLib(CCore* cx, char* path) {}

static inline void vt_addConsoleToast(CCore* self, char* msg) {}

static inline void vt_dguiDraw(CCore* self) {}

static inline void vt_dguiFrameBegin(CCore* self) {}

static inline void vt_dguiFrameFinish(CCore* self) {}

static inline void vt_dguiFrameDraw(CCore* self) {}

static inline bool vt_dguiIsOpen(CCore* self) {
    return false;
}

static inline void vt_dguiSetup(CCore* self, SdlWindow* window, void* sdlGlCtx) {}

static inline void vt_dguiShutdown(void) {}

/// Returns `true` if the developer GUI is open after the toggle.
static inline bool vt_dguiToggle(CCore* self) {
    return false;
}

static inline bool vt_dguiWantsKeyboard(CCore* self) {
    return false;
}

static inline bool vt_dguiWantsMouse(CCore* self) {
    return false;
}

static inline void vt_loadDynLibs(CCore* cx) {}

static inline bool vt_processEvent(CCore* self, SdlEvent* event) {
    return false;
}

static inline void vt_writeEngineTime(void) {}

/// Retrieve embedded window icon data.
const uint8_t* windowIcon(int32_t* size);

#endif // if !defined(RATBOOM_ZIG)
