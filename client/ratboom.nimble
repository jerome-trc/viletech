version = "0.0.0"
author = "jerome-trc"
description = "Personalized Doom source port forked from dsda-doom"
license = "Apache 2.0 OR MIT"
bin = @["src/main"]
skipDirs = @["tests"]

requires "nim == 2.0.4"
requires "https://github.com/jerome-trc/nimtie#7f07eee1"

import std/cmdline
import std/strformat

let pwd = getEnv("PWD")

proc build(release: static[bool], skipDsda: bool) =
    let libDirs = getEnv("VTEC_LIB_DIRS")
    var cmd = &"nim {libDirs} --cincludes:../../engine/src "

    for clib in [
        "dumb",
        "fluidsynth",
        "GL",
        "GLU",
        "mad",
        "ogg",
        "portmidi",
        "SDL2-2.0",
        "SDL2_image",
        "SDL2_mixer",
        "vorbis",
        "vorbisfile",
        "z",
        "zip",
    ]:
        cmd &= &"--clib:{clib} "

    when release:
        const outDir = "Release"
        cmd &= "--nimcache:../nimcache/release -d:release "
    else:
        const outDir = "Debug"
        cmd &= "--nimcache:../nimcache/debug --debuginfo --linedir:on "

    when defined(windows):
        cmd &= &"-o:../build/{outDir}/ratboom.exe "
    else:
        cmd &= &"-o:../build/{outDir}/ratboom "

    if skipDsda:
        echo("Skipping compilation of dsda-doom static library.")
    else:
        exec(&"cmake --build ../build --config {outDir} --target all --")

    cmd &= "cpp ./src/main.nim"
    exec(cmd)


task dbg, "Build Debug Executable":
    let params = commandLineParams()
    let skipDsda = "--skip:dsda" in params
    build(release = false, skipDsda)


task rel, "Build Release Executable":
    let params = commandLineParams()
    let skipDsda = "--skip:dsda" in params
    build(release = true, skipDsda)


task test, "Run Demo Tests":
    discard # TODO
