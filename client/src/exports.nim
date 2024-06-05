when defined(nimHasUsed):
    {.used.}

import std/[dynlib, paths]

import nimtie, nimtie/config

import core, devgui, platform

const afterIncludes = """
#ifndef NIMBASE_H // If included by Nim, don't expand to anything.

struct _NimString;
typedef struct _NimString Path;

typedef struct Core Core;
typedef void* LibHandle;
typedef union SdlEvent SdlEvent;
typedef struct SdlWindow SdlWindow;

"""

const header = """
/// @file
/// @brief Auto-generated C bindings to client functions. Do not edit.

"""

const cfg = Config(
    directory: "../nimcache",
    filename: "viletech.nim",
    header: header,
    targets: {Target.c},
    trailer: "\n#endif // ifndef NIMBASE_H\n",
    c: CConfig(
        afterIncludes: afterIncludes,
        cxxCompat: true,
        pragmaOnce: true,
        procPrefix: "vt_",
        structPrefix: "vt_",
    ),
)

exportEnums(cfg):
    Flavor

exportObject(cfg, CCore):
    discard

exportProcs(cfg):
    addDynLib
    dguiDraw
    dguiFrameBegin
    dguiFrameFinish
    dguiFrameDraw
    dguiNeedsMouse
    dguiSetup
    dguiShutdown
    dguiToggle
    loadDynLibs
    processEvent
    windowIcon

exportSeq(cfg, seq[LibHandle]):
    discard

exportSeq(cfg, seq[Path]):
    discard

when not defined(checkOnly):
    writeFiles(cfg)
