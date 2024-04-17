from std/cmdline import commandLineParams, paramCount
from std/envvars import getEnv
from std/parseopt import initOptParser, getopt
from std/os import nil
from std/paths import Path
from std/strformat import `&`
import std/[random, times]

import core, platform, stdx, wasmtime

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
var cx = initCore()

var clArgs = commandLineParams()
clArgs.insert(os.getAppFileName(), 0)

let argv = clArgs.toOpenArray(0, paramCount()).allocCStringArray()
let ret = dsdaMain(cx.ccorePtr(), paramCount().cint + 1, argv)

let uptime = startTime.elapsed().hoursMinsSecs()
echo(&"Engine uptime: {uptime.hours:02}:{uptime.mins:02}:{uptime.secs:02}")

quit(ret)
