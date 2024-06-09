## Permeates the code base with state that is practically "global".

import std/[dynlib, files]
from std/paths import Path

import devgui/console, imgui, plugin

const baseScreenWidth*: int = 320

type
    Flavor* {.pure.} = enum
        shareware
        registered
        commercial
        retail
        indeterminate
    DevGui* {.pure, size: sizeof(cint).} = enum
        console
        playground
        vfs
    Core* {.byref.} = object
        ## Permeates the code base with state that is practically "global".
        loadOrder*: seq[Path] = @[]
        dgui*: DGuiCore
        c*: CCore
    CCore* {.byref, exportc.} = object
        ## The parts of `Core` that can be safely exposed to C.
        core*: ptr Core = nil ## \
            ## Enable functions called from C to access the rest of the core.
        flavor*: Flavor = Flavor.indeterminate
        dynLibPaths*: seq[Path] = @[]
        dynLibs*: seq[LibHandle] = @[]
        savedGametick*: int32 = -1
    DGuiCore* = object
        imguiCtx*: ptr ImGuiContext = nil
        open*: bool = when defined(release): false else: true
        metricsWindow*: bool = false
        left*: DevGui = DevGui.console
        right*: DevGui = DevGui.vfs
        console*: Console

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


proc addConsoleToast*(self: var CCore, msg: cstring) {.exportc: "vt_$1".} =
    self.core.dgui.console.addToast($msg)


proc unloadDynLibs*(cx: var CCore) {.exportc: "vt_$1".} =
    for dylib in cx.dynLibs:
        unloadLib(dylib)

# Accessors ####################################################################

proc flavor*(self: Core): Flavor {.inline.} = self.c.flavor

proc `destroy=`*(self: Core) =
    discard
