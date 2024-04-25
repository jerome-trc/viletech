import trap, value

const hFunc = "<wasmtime/func.h>"
const hWasm = "<wasm.h>"

type
    WasmtimeCallerObj* {.header: hFunc, importc: "wasmtime_caller_t" } = object
    WasmtimeCaller* = ptr WasmtimeCallerObj
    WasmFuncTypeObj* {.header: hWasm, importc: "wasm_functype_t".} = object
    WasmFuncType* = ptr WasmFuncTypeObj

type WasmtimeFuncCallback* {.header: hFunc, importc: "wasmtime_func_callback_t".} = proc(
    env: pointer,
    caller: WasmtimeCaller,
    args: ptr WasmtimeVal,
    numArgs: csize_t,
    results: ptr WasmtimeVal,
    numResults: csize_t,
): WasmTrap {.cdecl.}

proc init*(_: typedesc[WasmFuncType], params, results: ptr WasmValTypeVec): WasmFuncType
    {.header: hWasm, importc: "wasm_functype_new".}
