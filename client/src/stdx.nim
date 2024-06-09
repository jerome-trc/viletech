## Helpers which could reasonably be a part of the standard library.

import std/times

proc clear*[T](s: seq[T]) =
    s.delete(0..(s.len - 1))

type HoursMinSecs* = tuple[hours: int64, mins: int64, secs: int64]
    ## Note that minutes and seconds are both remainders, not totals.

proc elapsed*(time: Time): Duration =
    getTime() - time


proc hoursMinsSecs*(duration: Duration): HoursMinSecs =
    let mins = (duration.inSeconds() / 60).int64
    let hours = (mins / 60).int64
    let secs = duration.inSeconds() mod 60
    return (hours, mins, secs)


proc cStr*(str: var string): cstring =
    return cast[cstring](str[0].addr)


proc toCstring*(str: string): cstring =
    let a = cast[ptr char](allocShared(str.len + 1))

    for i, c in str:
        let offs = cast[ptr char](cast[uint](a) + i.uint)
        offs[] = c

    return cast[cstring](a)


template bitFlags*(enumT: typedesc[enum], underlying: typedesc[SomeInteger]) =
    static:
        assert(sizeof(enumT) == sizeof(underlying))

    proc `+`*(a, b: enumT): enumT =
        cast[enumT](cast[underlying](a) or cast[underlying](b))

    proc `+=`*(a: var enumT, b: enumT) =
        a = a + b

    proc `-`*(a, b: enumT): enumT =
        cast[enumT](cast[underlying](a) and (not cast[underlying](b)))

    proc `-=`*(a: var enumT, b: enumT) =
        a = a - b

    proc `in`*(a, b: enumT): bool =
        let aU = cast[underlying](a)
        let bU = cast[underlying](b)
        return (aU and bU) != 0
