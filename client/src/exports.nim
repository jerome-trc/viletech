when defined(nimHasUsed):
    {.used.}

import std/[dynlib, paths]

import nimtie, nimtie/config

import core, devgui, platform

const afterIncludes = """
struct _NimString;
typedef struct _NimString Path;

typedef struct Core Core;
typedef void* LibHandle;
typedef union SdlEvent SdlEvent;
typedef struct SdlWindow SdlWindow;

"""

const cfg = Config(
    directory: "../build",
    filename: "viletech.nim",
    header: "/// @file\n/// @brief Auto-generated on " & gorge("date") & ".\n\n",
    targets: {Target.c},
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
    dguiSetup
    loadDynLibs
    processEvent
    windowIcon

exportSeq(cfg, seq[LibHandle]):
    discard

exportSeq(cfg, seq[Path]):
    discard

writeFiles(cfg)
