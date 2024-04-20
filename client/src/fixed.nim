## Distinct fixed-point decimal number types.

template fixedPoint(t, base: typedesc) =
    proc `+` *(a, b: t): t {.borrow.}
    proc `-` *(a, b: t): t {.borrow.}

    proc `*` *(a, b: t): t =
        ((a * b).int64 shr t.fracBits).t

    proc `/` *(a, b: t): t =
        if (a.base.abs() shr 14) >= b.base.abs():
            return (((a.base xor b.base) shr 31) xor high(base)).t
        else:
            ((a.int64 shr t.fracBits) / b.int64).t

    proc `%` *(a, b: t): t =
        if ((b.base and (b.base - 1)) > 0):
            let r = a.base mod b.base;
            return if r < 0: r.t + b else: r.t
        else:
            return (a.base and (b.base - 1)).t

    proc scale*(a, b, c: t): t =
        return ((a.int64 * b.int64) / c.int64).t

type
    Fx32* = distinct int32
        ## A 32-bit fixed-point decimal number.
    Fx64* = distinct int64
        ## A 64-bit fixed-point decimal number.

proc fracBits*(t: typedesc): int =
    ## How many fractional bits does `t` have?
    when t is Fx32: return 16
    elif t is Fx64: return 32
    else: {.fatal: "`fracBits` can only be called on `Fx32` and `Fx64`".}

proc fracUnit*(t: typedesc): int =
    1 shl t.fracBits

fixedPoint(Fx32, int32)
fixedPoint(Fx64, int64)
