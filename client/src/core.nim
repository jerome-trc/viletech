## Permeates the code base with state that is practically "global".

import std/[cmdline, dynlib, files]
from std/paths import Path

import devgui/console, flecs, imgui, plugin

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
        world*: World
    DGuiCore* = object
        imguiCtx*: ptr ImGuiContext = nil
        open*: bool = when defined(release): false else: true
        metricsWindow*: bool = false
        left*: DevGui = DevGui.console
        right*: DevGui = DevGui.vfs
        console*: Console

proc init*(_: typedesc[Core], argv: cstringArray): Core =
    var cx = Core()
    cx.c.world = World.initWithArgs(paramCount().cint + 1, argv)
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

proc flavor*(self {.byref.}: Core): Flavor {.inline.} = self.c.flavor
proc dynLibPaths*(self: var Core): var seq[Path] = self.c.dynLibPaths
proc dynLibs*(self: var Core): var seq[LibHandle] {.inline.} = self.c.dynLibs
proc savedGametick*(self {.byref.}: Core): int32 {.inline.} = self.c.savedGametick
proc world*(self {.byref.}: Core): World {.inline.} = self.c.world

proc `destroy=`*(self: Core) =
    assert(self.world.reset() == 0)
