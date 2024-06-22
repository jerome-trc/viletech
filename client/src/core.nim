## Permeates the code base with state that is practically "global".

import std/[cmdline, deques, dynlib, files, re]
from std/paths import Path

import flecs, imgui, plugin

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
    # Developer GUI ############################################################
    DevGui* {.pure, size: sizeof(cint).} = enum
        console
        playground
        vfs
    DGuiCore* = object
        imguiCtx*: ptr ImGuiContext = nil
        open*: bool = when defined(release): false else: true
        metricsWindow*: bool = false
        left*: DevGui = DevGui.console
        right*: DevGui = DevGui.vfs
        console*: Console
        vfs*: VfsGui
    ConsoleHistoryKind* {.pure.} = enum
        log
        submission
        toast
    ConsoleHistoryItem* = object
        case discrim*: ConsoleHistoryKind
        of log: log*: string
        of submission: submission*: string
        of toast: toast*: string
    Console* = object
        inputBuf*: array[256, char]
        history*: Deque[ConsoleHistoryItem]
        inputHistory*: Deque[string]
    VfsGui* = object
        filterBuf*: array[256, char]
        filter*: Regex

proc init*(_: typedesc[Core], argv: cstringArray): Core =
    var cx = Core()
    cx.c.world = World.initWithArgs(paramCount().cint + 1, argv)
    cx.dgui.vfs.filter = re("", {reIgnoreCase, reStudy})
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


proc addToHistory*(self: var Console, item: sink ConsoleHistoryItem) =
    if self.history.len > 1024:
        self.history.popFirst()

    self.history.addLast(item)


proc log*(self: var Console, msg: string) =
    echo(msg)
    self.addToHistory(ConsoleHistoryItem(discrim: ConsoleHistoryKind.log, log: msg))


# Accessors ####################################################################

proc console*(self: var Core): var Console {.inline.} = self.dgui.console
proc dynLibPaths*(self: var Core): var seq[Path] = self.c.dynLibPaths
proc dynLibs*(self: var Core): var seq[LibHandle] {.inline.} = self.c.dynLibs

proc flavor*(self {.byref.}: Core): Flavor {.inline.} = self.c.flavor
proc savedGametick*(self {.byref.}: Core): int32 {.inline.} = self.c.savedGametick
proc world*(self {.byref.}: Core): World {.inline.} = self.c.world

proc `destroy=`*(self: Core) =
    assert(self.world.reset() == 0)
