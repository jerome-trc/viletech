const hWasm = "<wasm.h>"
const hTrap = "<wasmtime/trap.h>"

type
    WasmTrapObj* {.header: hWasm, importc: "wasm_trap_t", incompleteStruct.} = object
    WasmTrap* = ptr WasmTrapObj

proc init*(_: typedesc[WasmTrap], msg: cstring, msgLen: csize_t)
    {.header: hTrap, importc: "wasmtime_trap_new".}

proc delete*(trap: WasmTrap)
    {.header: hWasm, importc: "wasm_trap_delete".}
