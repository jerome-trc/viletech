## Permeates the code base with state that is practically "global".

from std/paths import Path

const baseScreenWidth*: int = 320

type
    Flavor* {.pure.} = enum
        shareware
        registered
        commercial
        retail
        indeterminate
    Core* = object of CCore
        ## Permeates the code base with state that is practically "global".
        loadOrder*: seq[Path]
    CCore* {.exportc.} = object of RootObj
        ## The parts of `Core` that are FFI-safe.
        flavor*: Flavor = Flavor.indeterminate ## \
            ## It's not critical that this is always the first field,
            ## but please try to leave it that way.
        saved_gametick*: int32 = -1

proc init*(_: typedesc[Core]): Core =
    var cx = Core()
    return cx


proc ccorePtr*(cx: var Core): ptr CCore =
    let cxPtr: ptr Core = cx.addr
    let ccx: ptr CCore = cxPtr
    var ccxPtr: ptr CCore = nil

    for field in fields(ccx[]):
        ccxPtr = cast[ptr CCore](field.addr)
        break

    return ccxPtr


proc `destroy=`*(this: Core) =
    discard
