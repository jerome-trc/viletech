#!/usr/bin/env nim

import std/[strformat, strutils]

proc runDemo(
    iwad: string = "DOOM2.WAD",
    pwad: string = "",
    demo: string,
) =
    const cmdBase = "Release/ratboom -nosound -nodraw -levelstat -analysis"

    let pwadArg =
        if pwad.len == 0:
            ""
        else:
            "-file ../.temp/pwads/" & pwad

    exec(&"{cmdBase} -iwad ../.temp/iwads/{iwad} {pwadArg} -fastdemo ../sample/demos/{demo}")


proc expectAnalysis(key, val: string) =
    let analysis = staticRead("/home/jerome/Data/viletech/build/analysis.txt")

    for line in analysis.splitLines():
        let parts = line.split(" ")
        if parts[0] != key: continue
        assert(parts[1] == val)
        echo(parts[1], " vs. ", val)


proc expectTotalTime(val: string) =
    let levelStat = staticRead("/home/jerome/Data/viletech/build/levelstat.txt")
    let lines = levelStat.splitLines()
    let lastLine =
        if lines[lines.len - 1].len == 0: # Properly handle EOF newline
            lines[lines.len - 2]
        else:
            lines[lines.len - 1]

    let parts = lastLine.split(" ")
    assert(parts[3].strip(leading = true, trailing = true, {'(', ')'}) == val)


withDir("build"):
    runDemo(demo = "nm04-036.lmp")
    expectAnalysis("skill", "5")
    runDemo(pwad = "Valiant.wad", demo = "vae1-513.lmp")
    expectTotalTime("5:13")
