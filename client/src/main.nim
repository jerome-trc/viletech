from std/cmdline import commandLineParams, paramCount
from std/envvars import getEnv
from std/os import nil
from std/strformat import `&`
import std/[parseopt, random, times]

import core, fixed, platform, stdx

const libPath = when defined(release):
    "../build/src/Release/libratboom.a"
else:
    "../build/src/Debug/libratboom.a"

{.link: libPath.}
{.passc: "-I./src".}

{.link: getEnv("VTEC_WASMTIME_DIR") & "/target/release/libwasmtime.a".}
{.passc: "-isystem " & getEnv("VTEC_WASMTIME_DIR") & "/crates/c-api/include".}

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
            break
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
