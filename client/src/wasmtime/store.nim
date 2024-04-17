import engine

const hWasm = "<wasm.h>"
const hStore = "<wasmtime/store.h>"

type
    WasmtimeStoreObj* {.header: hStore, importc: "wasmtime_store_t", incompleteStruct.} = object
    WasmtimeStore* = ptr WasmtimeStoreObj

proc initStore*(
    this: WasmEngine,
    data: pointer,
    finalizer: proc(p: pointer): void {.cdecl.}
): WasmtimeStore
    {.header: hStore, importc: "wasmtime_store_new".}

proc delete*(store: WasmtimeStore)
    {.header: hStore, importc: "wasmtime_store_delete".}
