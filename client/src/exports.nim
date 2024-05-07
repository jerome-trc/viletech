when defined(nimHasUsed):
    {.used.}

from std/paths import Path

import nimtie, nimtie/config

import core, platform

const cfg = Config(
    directory: "../build",
    filename: "viletech.nim",
    header: "/// @file\n/// @brief Auto-generated on " & gorge("date") & ".\n\n",
    targets: {Target.c},
    c: CConfig(
        afterIncludes: "struct _NimString;\ntypedef struct _NimString Path;\n\n",
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
    windowIcon
    addDynLib

exportSeq(cfg, seq[Path]):
    discard

writeFiles(cfg)
