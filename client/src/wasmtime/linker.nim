import context, engine, error, function, instance, module, trap

const hLinker = "wasmtime/vtec-linker.h"

type
    WasmtimeLinkerObj* {.header: hLinker, importc: "wasmtime_linker_t".} = object
    WasmtimeLinker* = ptr WasmtimeLinkerObj

proc initLinker*(this: WasmEngine): WasmtimeLinker
    {.header: hLinker, importc: "wasmtime_linker_new".}

proc delete*(this: WasmtimeLinker)
    {.header: hLinker, importc: "wasmtime_linker_delete".}

proc allowShadowing*(this: WasmtimeLinker, allowed: bool)
    {.header: hLinker, importc: "wasmtime_linker_allow_shadowing".}

proc defineWasi*(this: WasmtimeLinker): WasmtimeError
    {.header: hLinker, importc: "wasmtime_linker_define_wasi".}

proc instantiate*(
    this: WasmtimeLinker,
    ctx: WasmtimeContext,
    module: WasmtimeModule,
    instance: ptr WasmtimeInstance,
    trap: ptr WasmTrap,
): WasmtimeError
    {.header: hLinker, importc: "wasmtime_linker_instantiate".}

proc defineFunc*(
    this: WasmtimeLinker,
    module: cstring,
    moduleLen: csize_t,
    funcName: cstring,
    nameLen: csize_t,
    funcType: WasmFuncType,
    cb: WasmtimeFuncCallback,
    data: pointer,
    finalizer: proc(data: pointer) {.cdecl.}
): WasmtimeError
    {.header: hLinker, importc: "wasmtime_linker_define_func".}
