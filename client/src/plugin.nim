## Types for callbacks that can be loaded from plugins.

type
    DynOnEngineInit* = proc() {.fastcall.}
