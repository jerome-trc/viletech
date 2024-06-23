## Helpers which could reasonably be a part of the standard library.

import std/[strformat, times]

type HoursMinSecs* = tuple[hours: int64, mins: int64, secs: int64]
    ## Note that minutes and seconds are both remainders, not totals.

proc hoursMinsSecs*(duration: Duration): HoursMinSecs =
    let mins = (duration.inSeconds() / 60).int64
    let hours = (mins / 60).int64
    let secs = duration.inSeconds() mod 60
    return (hours, mins, secs)


proc clear*[T](s: seq[T]) =
    s.delete(0..(s.len - 1))


proc cStr*(str {.byref.}: string): cstring =
    return cast[cstring](str[0].addr)


proc elapsed*(time: Time): Duration =
    getTime() - time


proc isEmpty*[T](sequence: seq[T]): bool =
    sequence.len() == 0


proc isEmpty*(str: string): bool =
    str.len() == 0


proc isEmpty*[TOpenArray: openArray | varargs](arr: TOpenArray): bool =
    arr.len() == 0


proc offset*[T](self: ptr T, count: int): ptr T =
    let p = cast[uint](self)
    return cast[ptr T](p + (count * sizeof(T)).uint)


proc subdivideBytes*(size: int): string =
    if size == 0:
        return "0 B"

    var s = size.float
    var unit = "B"

    if s > 1024.0:
        s /= 1024.0
        unit = "KB"
    else:
        return &"{s:.2f} {unit}"

    if s > 1024.0:
        s /= 1024.0
        unit = "MB"

    if s > 1024.0:
        s /= 1024.0
        unit = "GB"

    return &"{s:.2f} {unit}"


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

#[

macro bitFlags2*(name: untyped, public, pure: static[bool], body: untyped) =
    var ret = nnkStmtList.newTree()
    var fields: seq[NimNode] = @[]
    var prevFlags = initTable[string]()

    for i, section in body:
        var f = new newNimNode(nnkEnumFieldDef)
        f.add(section)

        if section.kind == nnkIdent:
            f.add(newIntLitNode(1 shl i))
        elif section.kind == nnkAsgn:
            echo section.treeRepr

        fields.add(f)

    ret.add(newEnum(name, fields, public, pure))
    echo ret.toStrLit
    return ret

]#
