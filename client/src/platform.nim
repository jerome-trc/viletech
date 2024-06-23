import std/[files, syncio, paths, parseutils, times]

import imports, stdx

var startTime* {.global.}: Time

proc windowIcon*(size: var int32): ptr uint8 {.exportc: "vt_$1".} =
    ## Retrieve embedded window icon data.
    const bytes = staticRead("../../engine/ICONS/viletech.png")
    let b = cast[seq[uint8]](bytes)
    size = b.len.int32
    return b[0].addr


proc writeEngineTime*() {.exportc: "vt_$1".} =
    let seconds = startTime.elapsed.inSeconds
    let timeFilePath = ($doomExeDir()).Path / "time.txt".Path

    if not timeFilePath.fileExists:
        try:
            writeFile(timeFilePath.string, $seconds)
        except:
            echo("Failed to create file: " & timeFilePath.string)
        return

    var file = try:
        open(timeFilePath.string)
    except:
        echo("Failed to open for reading: " & timeFilePath.string)
        return

    let prevText = try:
        file.readAll()
    except:
        echo("Failed to read contents: " & timeFilePath.string)
        return

    var prevSeconds: int
    let charsProcessed = try:
        prevText.parseInt(prevSeconds)
    except:
        echo("File's contents are not a valid integer: " & timeFilePath.string)
        return

    if charsProcessed < 1:
        echo("File's contents are not a valid integer: " & timeFilePath.string)
        return

    if not file.reopen(timeFilePath.string, fmWrite):
        echo("Faile to open for writing: " & timeFilePath.string)
        return

    try:
        file.write($(prevSeconds + seconds))
    except:
        echo("Failed to write to: " & timeFilePath.string)
