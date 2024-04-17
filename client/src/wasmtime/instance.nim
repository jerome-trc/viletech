import context, error, extern, module, trap

const hInstance = "<wasmtime/instance.h>"

type
    WasmtimeInstance* {.header: hInstance, importc: "wasmtime_instance_t", incompleteStruct.} = object
        storeId*: uint64
        index*: csize_t

proc initInstance*(
    this: WasmtimeContext,
    module: WasmtimeModule,
    extern: ptr WasmtimeExtern,
    numImports: csize_t,
    instance: ptr WasmtimeInstance,
    trap: ptr WasmTrap
): WasmtimeError
    {.header: hInstance, importc: "wasmtime_instance_new".}

proc exportGet*(
    this: WasmtimeContext,
    inst: ptr WasmtimeInstance,
    name: cstring,
    nameLen: csize_t,
    item: ptr WasmtimeExtern
): bool
    {.header: hInstance, importc: "wasmtime_instance_export_get".}
