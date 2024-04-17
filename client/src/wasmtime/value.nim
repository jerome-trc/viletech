const hWasm = "<wasm.h>"
const hVal = "<wasmtime/val.h>"

type
    WasmRefObj* {.header: hWasm, importc: "wasm_ref_t", incompleteStruct} = object
    WasmRef* = ptr WasmRefObj
    WasmtimeV128* = array[16, uint8]
    WasmtimeValKind* {.
        header: hVal,
        importc: "wasmtime_valkind_t",
        size: sizeof(uint8)
    .} = enum
        wasmvI32,
        wasmvI64,
        wasmvF32,
        wasmvF64,
        wasmvRef,
    WasmtimeValUnion* {.header: hVal, importc: "wasmtime_valunion_t", union.} = object
        i32*: int32
        i64*: int64
        f32*: float32
        f64*: float64
        anyRef*: pointer
        externRef*: pointer
        funcRef*: pointer
        v128*: WasmtimeV128
    WasmtimeVal* {.header: hVal, importc: "wasmtime_val_t".} = object
        kind*: WasmtimeValKind
        `of`*: WasmtimeValUnion
