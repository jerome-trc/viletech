## Permeates the code base with state that is practically "global".

import std/[dynlib, files]
from std/paths import Path

import plugin

const baseScreenWidth*: int = 320

type
    Flavor* {.pure.} = enum
        shareware
        registered
        commercial
        retail
        indeterminate
    Core* {.byref.} = object
        ## Permeates the code base with state that is practically "global".
        loadOrder*: seq[Path]
        c*: CCore
    CCore* {.byref, exportc.} = object
        ## The parts of `Core` that are FFI-safe.
        core*: ptr Core = nil ## \
            ## Enable functions called from C to access the rest of the core.
        flavor*: Flavor = Flavor.indeterminate ## \
            ## It's not critical that this is always the first field,
            ## but please try to leave it that way.
        dynLibPaths*: seq[Path] = @[]
        dynLibs*: seq[LibHandle] = @[]
        savedGametick*: int32 = -1

proc init*(_: typedesc[Core]): Core =
    var cx = Core()
    return cx


proc addDynLib*(cx: var CCore, path: cstring) {.exportc: "vt_$1".} =
    cx.dynLibPaths.add(($path).Path)
    echo("Registering plugin: " & cx.dynLibPaths[cx.dynLibPaths.len - 1].string)


proc loadDynLibs*(cx: var CCore) {.exportc: "vt_$1".} =
    for dylibPath in cx.dynLibPaths:
        if not dylibPath.fileExists():
            continue # TODO: report an error.

        let dylib = loadLib(dylibPath.string)

        if dylib.isNil:
            continue # TODO: report an error.

        cx.dynLibs.add(dylib)
        let onEngineInit = cast[DynOnEngineInit](dylib.symAddr(cstring"onEngineInit"))

        if onEngineInit.isNil:
            continue

        onEngineInit()


proc unloadDynLibs*(cx: var CCore) {.exportc: "vt_$1".} =
    for dylib in cx.dynLibs:
        unloadLib(dylib)

# Accessors ####################################################################

proc flavor*(this: Core): Flavor {.inline.} = this.c.flavor

proc `destroy=`*(this: Core) =
    discard
