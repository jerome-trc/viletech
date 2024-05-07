when defined(nimHasUsed):
    {.used.}

import std/[dynlib, paths]

import nimtie, nimtie/config

import core, platform

const afterIncludes = """
struct _NimString;
typedef struct _NimString Path;

typedef struct Core Core;
typedef void* LibHandle;
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
    loadDynLibs
    windowIcon

exportSeq(cfg, seq[LibHandle]):
    discard

exportSeq(cfg, seq[Path]):
    discard

writeFiles(cfg)
