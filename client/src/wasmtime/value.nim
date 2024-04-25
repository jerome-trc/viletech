import vec

const hWasm = "<wasm.h>"
const hVal = "<wasmtime/val.h>"

type
    WasmRefObj* {.header: hWasm, importc: "wasm_ref_t", incompleteStruct} = object
    WasmRef* = ptr WasmRefObj
    WasmValKind* {.
        header: hWasm,
        importc: "wasm_valkind_t",
        size: sizeof(uint8)
    .} = enum
        wasmvI32,
        wasmvI64,
        wasmvF32,
        wasmvF64,
        wasmvExtRef = 128,
        wasmvFuncRef
    WasmtimeValKind* {.
        header: hVal,
        importc: "wasmtime_valkind_t",
        size: sizeof(uint8)
    .} = enum
        wasmtvI32,
        wasmtvI64,
        wasmtvF32,
        wasmtvF64,
        wasmtvRef,
    WasmtimeV128* = array[16, uint8]
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
    WasmValTypeObj* {.header: hWasm, importc: "".} = object
    WasmValType* = ptr WasmValTypeObj

proc init*(_: typedesc[WasmValType], v: WasmValKind): WasmValType
    {.header: hWasm, importc: "wasm_valtype_new".}

proc kind*(this: WasmValType): WasmValKind
    {.header: hWasm, importc: "wasm_valtype_kind".}

declareWasmtimeVec(WasmValTypeVec, WasmValType, valtype)
