import std/[deques, parseopt, tables, unicode]

import ../[core, imgui, stdx]

import ccmds

type
    HistoryKind = ConsoleHistoryKind
    HistoryItem = ConsoleHistoryItem

const commands = {
    "exit": (cmd: "exit", fn: ccmdExit),
    "music.play": (cmd: "music.play", fn: ccmdMusicPlay),
    "quit": (cmd: "quit", fn: ccmdExit),
}.toTable

proc inputTextCallback(data: ptr ImGuiInputTextCallbackData): cint {.noconv.}
proc submit(self: var Core)

proc draw*(self: var Core, left: bool, menuBarHeight: float32) =
    let vp = imGuiGetMainViewport()

    if left:
        imGuiSetNextWindowPos(ImVec2(y: menuBarHeight))
    else:
        imGuiSetNextWindowPos(ImVec2(x: vp.size.x * 0.5, y: menuBarHeight))

    imGuiSetNextWindowSize(ImVec2(x: vp.size.x * 0.5, y: vp.size.y * 0.33))

    if not imGuiBeginWindow(
        cstring"Console",
        flags = ImGuiWindowFlags.noTitleBar + ImGuiWindowFlags.noResize
    ):
        return

    defer: imGuiEndWindow()

    let footerHeightToReserve =
        ImGuiStyle.get().itemSpacing.y +
        imGuiGetFrameHeightWithSpacing()

    if imGuiBeginChild(
        cstring"scrollingRegion",
        ImVec2(x: 0.0, y: -footerHeightToReserve),
        ImGuiChildFlags.none,
        ImGuiWindowFlags.horizontalScrollbar
    ):
        defer: imGuiEndChild()

        var clipper = ImGuiListClipper.init()
        clipper.begin(self.console.history.len.cint)

        while clipper.step():
            for i in clipper.displayStart ..< clipper.displayEnd:
                case self.console.history[i].discrim:
                of HistoryKind.log:
                    imGuiTextUnformatted(self.console.history[i].log.cStr())
                of HistoryKind.submission:
                    imGuiTextUnformatted(self.console.history[i].submission.cStr())
                of HistoryKind.toast:
                    imGuiTextUnformatted(self.console.history[i].toast.cStr())

    if imGuiInputText(
        cstring"##console.inputBuf",
        self.console.inputBuf[0].addr,
        self.console.inputBuf.len.csize_t,
        flags =
            ImGuiInputTextFlags.callbackCompletion +
            ImGuiInputTextFlags.callbackHistory +
            ImGuiInputTextFlags.enterReturnsTrue,
        callback = inputTextCallback
    ):
        self.submit()

    imGuiSameLine()

    if imGuiButton(cstring"Submit"):
        self.submit()

    imGuiSetItemDefaultFocus()


proc addConsoleToast*(self: var CCore, msg: cstring) {.exportc: "vt_$1".} =
    self.core[].console.addToHistory(HistoryItem(discrim: HistoryKind.toast, toast: $msg))


# Internal #####################################################################

proc inputTextCallback(data: ptr ImGuiInputTextCallbackData): cint {.noconv.} =
    echo($data.eventFlag)
    return 0


proc submit(self: var Core) =
    let submission = $cast[cstring](self.console.inputBuf[0].addr)

    if submission.len < 1:
        echo("$")
        return

    if self.console.inputHistory.len > 256:
        self.console.inputHistory.popFirst()

    if self.console.inputHistory.len < 1 or self.console.inputHistory.peekLast != submission:
        self.console.inputHistory.addLast(submission)

    defer: self.console.inputBuf = default(typeof(self.console.inputBuf))
    let logged = "$ " & submission
    echo(logged)
    self.console.addToHistory(
        HistoryItem(discrim: HistoryKind.submission, submission: logged)
    )

    let halves = submission.split(maxsplit = 1)
    let cmdName = halves[0]

    let ccmd = try:
        commands[cmdName]
    except:
        self.console.log(cmdName & ": command not found")
        return

    var args = if halves.len >= 2:
        initOptParser(halves[1])
    else:
        initOptParser("")

    ccmd.fn(self, args)
