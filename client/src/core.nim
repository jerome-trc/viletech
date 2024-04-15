## Permeates the code base with state that is practically "global".

from std/paths import Path

import wasmtime

const baseScreenWidth*: int = 320

type
    Flavor* {.pure.} = enum
        shareware
        registered
        commercial
        retail
        indeterminate
    Core* = ref object of CCore
        ## Permeates the code base with state that is practically "global".
        loadOrder*: seq[Path]
    CCore* {.exportc.} = object of RootObj
        ## The parts of `Core` that are FFI-safe.
        flavor*: Flavor = Flavor.indeterminate ## \
            ## It's not critical that this is always the first field,
            ## but please try to leave it that way.
        saved_gametick*: int32 = -1
        wasm*: WasmEngine = nil

proc `destroy=`*(this: Core) =
    discard
