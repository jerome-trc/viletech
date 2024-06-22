## Console command functions.

import std/parseopt

import ../[core, imports, stdx]

proc ccmdExit*(cx: var Core, args: var OptParser) =
    clearMenus()


proc ccmdMusicPlay*(cx: var Core, args: var OptParser) =
    var name = ""
    var looping = true

    while true:
        args.next()
        case args.kind
        of cmdArgument:
            name = args.key
        of cmdLongOption:
            if args.key == "" and args.val == "":
                break # `--`
            elif args.key == "once":
                looping = false
            else:
                cx.console.log(args.key & ": unknown parameter")
                return
        of cmdShortOption:
            if args.key == "o":
                looping = false
            else:
                cx.console.log(args.key & ": unknown parameter")
                return
        of cmdEnd: break

    while true:
        args.next()
        case args.kind
        of cmdArgument, cmdLongOption, cmdShortOption:
            name = args.key
        of cmdEnd: break

    if name.len < 1:
        cx.console.log("The name of a music lump must be provided.")
        return

    if not looping:
        cx.console.log("Song will not be looped.")

    if not changeMusicByName(cx.c.addr, name.cStr, looping):
        cx.console.log(name & ": music lump not found.")
    else:
        cx.console.log("Now playing: " & name)
