const hError = "<wasmtime/error.h>"

type
    WasmtimeErrorObj* {.header: hError, importc: "wasmtime_error_t".} = object
    WasmtimeError* = ptr WasmtimeErrorObj

proc init*(_: typedesc[WasmtimeError], msg: cstring): WasmtimeError
    {.header: hError, importc: "wasmtime_error_new".}

proc delete*(this: WasmtimeError)
    {.header: hError, importc: "wasmtime_error_delete".}
