from std/cmdline import commandLineParams, paramCount
from std/os import nil
from std/strformat import `&`
import std/[parseopt, random, times]

import core, exports, fixed, imports, platform, stdx

{.passc: "-I./src".}

const dsdaLibPath = when defined(release):
    "../build/src/Release/libratboom.a"
else:
    "../build/src/Debug/libratboom.a"

{.link: dsdaLibPath.}

const projectDir* {.strdefine.} = "."
    ## i.e. `viletech/client`.

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

var cx = Core.init()

clArgs.insert(os.getAppFileName(), 0)
let argv = clArgs.toOpenArray(0, paramCount()).allocCStringArray()
let ret = dsdaMain(cx.ccorePtr(), paramCount().cint + 1, argv)

let uptime = startTime.elapsed().hoursMinsSecs()
echo(&"Engine uptime: {uptime.hours:02}:{uptime.mins:02}:{uptime.secs:02}")

quit(ret)
