import error, extern, store, trap, value, wasi

const hFunc = "<wasmtime/func.h>"
const hStore = "<wasmtime/store.h>"

type
    WasmtimeContextObj* {.header: hStore, importc: "wasmtime_context_t", incompleteStruct.} = object
    WasmtimeContext* = ptr WasmtimeContextObj

proc context*(store: WasmtimeStore): WasmtimeContext
    {.header: hStore, importc: "wasmtime_store_context".}

proc setWasi*(this: WasmtimeContext, wasi: WasiConfig): WasmtimeError
    {.header: hStore, importc: "wasmtime_context_set_wasi".}

proc funcCall*(
    this: WasmtimeContext,
    fn: ptr WasmtimeFunc,
    args: ptr WasmtimeVal,
    numArgs: csize_t,
    results: ptr WasmtimeVal,
    numResults: csize_t,
    trap: ptr WasmTrap
): WasmtimeError
    {.header: hFunc, importc: "wasmtime_func_call".}
