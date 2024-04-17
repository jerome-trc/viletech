const hWasm = "<wasm.h>"
const hEngine = "<wasmtime/engine.h>"

type
    WasmConfigObj* {.header: hWasm, importc: "wasm_config_t", incompleteStruct.} = object
    WasmConfig* = ptr WasmConfigObj
    WasmEngineObj* {.header: hWasm, importc: "wasm_engine_t", incompleteStruct.} = object
    WasmEngine* = ptr WasmEngineObj

proc init*(_: typedesc[WasmEngine]): WasmEngine
    {.header: hWasm, importc: "wasm_engine_new".}

proc initWithConfig*(_: typedesc[WasmEngine], config: WasmConfig): WasmEngine
    {.header: hWasm, importc: "wasm_engine_new_with_config"}

proc delete*(this: WasmEngine)
    {.header: hWasm, importc: "wasm_engine_delete".}

proc incEpoch*(this: WasmEngine)
    {.header: hEngine, importc: "wasmtime_engine_increment_epoch".}

proc init*(_: typedesc[WasmConfig]): WasmConfig
    {.header: hWasm, importc: "wasm_config_new".}

proc delete*(this: WasmConfig)
    {.header: hWasm, importc: "wasm_config_delete".}
