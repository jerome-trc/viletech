proc windowIcon*(size: var int32): ptr uint8 {.exportc: "vt_$1".} =
    ## Retrieve embedded window icon data.
    const bytes = staticRead("../../engine/ICONS/viletech.png")
    let b = cast[seq[uint8]](bytes)
    size = b.len.int32
    return b[0].addr
