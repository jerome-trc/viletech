import vec

const hError = "<wasmtime/error.h>"

type
    WasmtimeErrorObj* {.header: hError, importc: "wasmtime_error_t".} = object
    WasmtimeError* = ptr WasmtimeErrorObj

proc init*(_: typedesc[WasmtimeError], msg: cstring): WasmtimeError
    {.header: hError, importc: "wasmtime_error_new".}

proc delete*(this: WasmtimeError)
    {.header: hError, importc: "wasmtime_error_delete".}

proc message*(this: WasmtimeError): string =
    proc impl(this: WasmtimeError, msg: ptr WasmName)
        {.header: hError, importc: "wasmtime_error_message".}

    var m = WasmName()
    m.addr.initEmpty()
    impl(this, m.addr)
    let c = cast[cstring](m.data)
    let ret = $c
    m.addr.delete()
    return ret
