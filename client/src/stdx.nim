## Helpers which could reasonably be a part of the standard library.

import std/times

type HoursMinSecs* = tuple[hours: int64, mins: int64, secs: int64]
    ## Note that minutes and seconds are both remainders, not totals.

proc elapsed*(time: Time): Duration =
    getTime() - time


proc hoursMinsSecs*(duration: Duration): HoursMinSecs =
    let mins = (duration.inSeconds() / 60).int64
    let hours = (mins / 60).int64
    let secs = duration.inSeconds() mod 60
    return (hours, mins, secs)


proc toCstring*(str: string): cstring =
    let a = cast[ptr char](allocShared(str.len + 1))

    for i, c in str:
        let offs = cast[ptr char](cast[uint](a) + i.uint)
        offs[] = c

    return cast[cstring](a)
