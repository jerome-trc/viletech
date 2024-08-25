#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include <SDL.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

struct CCore;

void addConsoleToast(struct CCore*, const char*);

void dguiLayout(struct CCore*);

void dguiFrameBegin(struct CCore*);

void dguiFrameFinish(struct CCore*);

void dguiFrameDraw(struct CCore*);

void dguiSetup(struct CCore*, SDL_Window*, void* sdl_gl_ctx);

void dguiShutdown(void);

bool dguiWantsKeyboard(struct CCore*);

bool dguiWantsMouse(struct CCore*);

bool dguiProcessEvent(struct CCore*, SDL_Event*);

void populateMusicPlayer(struct CCore*);

#endif // if !defined(RATBOOM_ZIG)
