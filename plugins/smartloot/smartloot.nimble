version = "0.0.0"
author = "jerome-trc"
description = "Plugin for RatBoom"
license = "Apache 2.0 OR MIT"
bin = @["src/main"]
skipDirs = @["tests"]

requires "nim == 2.0.4"

import std/strformat

proc build(release: static[bool]) =
    var cmd = "nim --app:lib "
    cmd &= "-p:../../client/src "

    when release:
        const outDir = "Release"
        cmd &= "--nimcache:../../nimcache/plugins/smartloot/debug "
        cmd &= "-d:release "
    else:
        const outDir = "Debug"
        cmd &= "--nimcache:../../nimcache/plugins/smartloot/release "
        cmd &= "--debuginfo --linedir:on "

    let libName = toDll("smartloot")
    cmd &= &"-o:../../build/{outDir}/{libName} "

    cmd &= "c ./src/lib.nim"
    exec(cmd)


task dbg, "Build Unoptimized Library":
    build(release = false)


task rel, "Build Optimized Library":
    build(release = true)
