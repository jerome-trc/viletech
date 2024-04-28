when defined(nimHasUsed):
    {.used.}

import nimtie, nimtie/config

import core, platform

const cfg = Config(
    directory: "../build",
    filename: "viletech.nim",
    targets: {Target.c},
    c: CConfig(
        pragmaOnce: true,
        procPrefix: "vt",
        structPrefix: "vt",
    ),
)

exportEnums:
    Flavor

exportObject CCore:
    discard

exportProcs:
    windowIcon

writeFiles(cfg)
