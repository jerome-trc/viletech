import std/[macros, strformat]

const hWasm = "<wasm.h>"

macro declareWasmtimeVec*(name, element, cElemName: untyped): untyped =
    let cname = newLit(&"wasm_{cElemName.repr}_vec_t")
    let cnameInitEmpty = newLit(&"wasm_{cElemName.repr}_vec_new_empty")
    let cnameInitUninit = newLit(&"wasm_{cElemName.repr}_vec_new_uninitialized")
    let cnameDelete = newLit(&"wasm_{cElemName.repr}_vec_delete")
    let cnameCopy = newLit(&"wasm_{cElemName.repr}_vec_copy")

    return quote do:
        type `name`* {.header: hWasm, importc: `cname`.} = object
            size*: csize_t
            data*: ptr `element`

        proc initEmpty*(this: ptr `name`)
            {.header: hWasm, importc: `cnameInitEmpty`.}

        proc initUninit*(this: ptr `name`, len: csize_t)
            {.header: hWasm, importc: `cnameInitUninit`.}

        proc delete*(this: ptr `name`)
            {.header: hWasm, importc: `cnameDelete`.}

        proc copy*(this, other: ptr `name`)
            {.header: hWasm, importc: `cnameCopy`.}

        proc setAt*(this: var `name`, i: int, elem: `element`) =
            let offs = cast[uint](this.data) + (i.uint * sizeof(`element`).uint)
            let p = cast[ptr `element`](offs)
            p[] = elem

declareWasmtimeVec(WasmName, byte, byte)
