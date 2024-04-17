const hExtern = "<wasmtime/extern.h>"

type
    WasmtimeFunc* {.header: hExtern, importc: "wasmtime_func_t".} = object
        storeId*: uint64
        index*: csize_t
    WasmtimeGlobal* {.header: hExtern, importc: "wasmtime_global_t".} = object
        storeId*: uint64
        index*: csize_t
    WasmtimeMemory* {.header: hExtern, importc: "wasmtime_memory_t".} = object
        storeId*: uint64
        index*: csize_t
    WasmtimeTable* {.header: hExtern, importc: "wasmtime_table_t".} = object
        storeId*: uint64
        index*: csize_t
    WasmtimeExternKind* {.
        header: hExtern,
        importc: "wasmtime_extern_kind_t",
        size: sizeof(uint8)
    .} = enum
        wexkFunc = 0,
        wexkGlobal = 1,
        wexkTable = 2,
        wexkMemory = 3,
        wexkSharedMemory = 4,
    WasmtimeExternUnion* {.
        header: hExtern,
        importc: "wasmtime_extern_union_t",
        union
    .} = object
        `func`*: WasmtimeFunc
        global*: WasmtimeGlobal
        table*: WasmtimeTable
        memory*: WasmtimeMemory
        sharedMem*: pointer
    WasmtimeExtern* {.header: hExtern, importc: "wasmtime_extern_t".} = object
        kind*: WasmtimeExternKind
        `of`*: WasmtimeExternUnion
