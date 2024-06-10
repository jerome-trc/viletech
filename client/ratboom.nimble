version = "0.0.0"
author = "jerome-trc"
description = "Personalized Doom source port forked from dsda-doom"
license = "Apache 2.0 OR MIT"
bin = @["ratboom"]
skipDirs = @["tests"]

requires "nim == 2.0.4"
requires "checksums == 0.1.0" # Only used in this file.
requires "https://github.com/jerome-trc/nimtie#c24b804"

import std/[cmdline, strformat]
import checksums/md5

proc build(release: static[bool], checkOnly: bool, skipDsda: bool) =
    let libDirs = getEnv("VTEC_LIB_DIRS")
    var cmd = &"nim {libDirs} --cincludes:../engine/src "
    cmd &= "--cincludes:../depend/imgui --cincludes:../depend/flecs "

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
        cmd &= &"--nimcache:../nimcache/release -d:release -d:strip -d:lto "
    else:
        const outDir = "Debug"
        # https://github.com/nim-lang/Nim/issues/22824
        # cmd &= "--hotCodeReloading:on -d:nimDebugDlOpen --mm:refc "
        cmd &= &"--nimcache:../nimcache/debug --debuginfo --lineDir:on "

    let exeName = toExe("ratboom")
    cmd &= &"-o:../build/{outDir}/{exeName} "

    if checkOnly:
        cmd &= "--compileOnly:on -d:checkOnly "

    if skipDsda:
        echo("Skipping compilation of dsda-doom static library.")
    else:
        exec(&"cmake --build ../build --config {outDir} --target all --")

    cmd &= &"cpp -d:projectDir:{getCurrentDir()} ./ratboom.nim"
    exec(cmd)

    # If the generated C header isn't different from the last run, don't copy it
    # so as not to cause Ninja cache invalidation and force a rebuild of dsda-doom.
    if "../build/viletech.nim.h".fileExists():
        let prevBindings = staticRead("../build/viletech.nim.h")
        let prevCksum = toMD5(prevBindings)
        let newBindings = staticRead("../nimcache/viletech.nim.h")
        let newCksum = toMD5(newBindings)

        if prevCksum != newCksum:
            cpFile("../nimcache/viletech.nim.h", "../build/viletech.nim.h")
    else:
        cpFile("../nimcache/viletech.nim.h", "../build/viletech.nim.h")


task dbg, "Build Debug Executable":
    let params = commandLineParams()
    let skipDsda = "--skip:dsda" in params
    build(release = false, checkOnly = false, skipDsda)


task rel, "Build Release Executable":
    let params = commandLineParams()
    build(release = true, checkOnly = false, false)


task semchk, "Compiler Check":
    let params = commandLineParams()
    build(release = false, checkOnly = true, true)


task test, "Run Demo Tests":
    discard # TODO
