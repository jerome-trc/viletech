## Functions from dsda-doom's C and C++ code, exposed to Nim.

const hLPrintF = "lprintf.h"

type OutputLevels* {.size: sizeof(cint).} = enum
    outlvInfo = 1,
    outlvWarn = 2,
    outlvError = 4,
    outlvDebug = 8,

proc lprintf*(lvl: OutputLevels, fmt: cstring)
    {.importc: "lprintf", varargs, cdecl.}

proc iWarn*(error: cstring)
    {.header: hLPrintF, importc: "I_Warn", varargs, cdecl.}
