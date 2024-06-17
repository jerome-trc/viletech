## Wrapper around libgamemode.

import std/dynlib

type
    ProcessId* = distinct int32
    StateDiscriminator = enum
        sUninit
        sInit
        sFailed
    SymTable = object
        dylib: LibHandle
        requestStart, requestEnd, queryStatus:
            proc(): cint {.cdecl.}
        requestStartFor, requestEndFor, queryStatusFor:
            proc(pid: ProcessId): cint {.cdecl.}
        errorString:
            proc(): cstring {.cdecl.}
    StateCase = object
        case discrim: StateDiscriminator
        of sUninit: discard
        of sFailed: discard
        of sInit: symTable: SymTable

var state {.global.}: StateCase = StateCase(discrim: sUninit)

when defined(linux):
    proc gameModeStart*() =
        case state.discrim:
        of sUninit: discard
        of sInit, sFailed: return

        var dylib = loadLib("libgamemode.so.0")

        if dylib == nil:
            dylib = loadLib("libgamemode.so")

        if dylib == nil:
            raise newException(IOError, "libgamemode not found")

        var symTable = SymTable()

        try:
            symTable.errorString = cast[proc(): cstring {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_error_string")
            )

            symTable.requestStart = cast[proc(): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_request_start")
            )
            symTable.requestEnd = cast[proc(): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_request_end")
            )
            symTable.queryStatus = cast[proc(): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_query_status")
            )

            symTable.requestStartFor = cast[proc(_: ProcessId): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_request_start_for")
            )
            symTable.requestEndFor = cast[proc(_: ProcessId): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_request_end_for")
            )
            symTable.queryStatusFor = cast[proc(_: ProcessId): cint {.cdecl.}](
                dylib.checkedSymAddr(cstring"real_gamemode_query_status_for")
            )
        except LibraryError as err:
            echo("Failed to retrieve libgamemode functions: " & err.msg)
            return

        state = StateCase(discrim: sInit, symTable: symTable)

        if state.symTable.requestStart() < 0:
            raise newException(CatchableError, "GameMode failed to start")


else:
    proc gameModeStart*() = discard

