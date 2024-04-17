import engine, error

const hModule = "<wasmtime/module.h>"

type
    WasmtimeModuleObj* {.header: hModule, importc: "wasmtime_module_t", incompleteStruct.} = object
    WasmtimeModule* = ptr WasmtimeModuleObj

proc initModule*(
    this: WasmEngine,
    wasm: ptr uint8,
    wasmLen: csize_t,
    ret: ptr WasmtimeModule
): WasmtimeError
    {.header: hModule, importc: "wasmtime_module_new".}

proc delete*(module: WasmtimeModule)
    {.header: hModule, importc: "wasmtime_module_delete".}

proc validate*(engine: WasmEngine, wasm: ptr uint8, wasmLen: csize_t): WasmtimeError
    {.header: hModule, importc: "wasmtime_module_validate"}
