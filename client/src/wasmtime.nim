const wasm = "<wasm.h>"

type
    WasmEngineObj* {.header: wasm, importc: "wasm_engine_t".} = object
    WasmEngine* = ptr WasmEngineObj

proc initWasmEngine*(): WasmEngine {.importc: "wasm_engine_new".}
