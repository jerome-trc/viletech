from std/cmdline import commandLineParams, paramCount
from std/os import nil
from std/strformat import `&`
import std/[parseopt, random, times]

import compile, core, exports, fixed, gamemode, imports, platform, stdx

proc dsdaMain(
    ccx: ptr CCore,
    argc: cint,
    argv: cstringArray
): cint {.importc.}

# Actual code starts here ######################################################

let startTime = getTime()
randomize()

var clArgs = commandLineParams()
var optParser = initOptParser(clArgs)

while true:
    optParser.next()
    case optParser.kind
    of cmdArgument: discard
    of cmdLongOption:
        if optParser.key == "" and optParser.val == "":
            break # `--`
    of cmdShortOption: discard
    of cmdEnd: break

# Everything after `--`. Not sure what to do with these yet...

while true:
    optParser.next()
    case optParser.kind
    of cmdArgument: discard
    of cmdLongOption: discard
    of cmdShortOption: discard
    of cmdEnd: break

gameModeStart()

var cx = Core.init()
cx.c.core = cx.addr

clArgs.insert(os.getAppFileName(), 0)
let argv = clArgs.toOpenArray(0, paramCount()).allocCStringArray()
let ret = dsdaMain(cx.c.addr, paramCount().cint + 1, argv)

let uptime = startTime.elapsed().hoursMinsSecs()
echo(&"Engine uptime: {uptime.hours:02}:{uptime.mins:02}:{uptime.secs:02}")

quit(ret)
