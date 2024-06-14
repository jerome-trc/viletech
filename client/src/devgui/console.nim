import std/deques

import ../[imgui, stdx]

type
    HistoryKind {.pure.} = enum
        log
        submission
        toast
    HistoryItem = object
        case discrim: HistoryKind
        of log: log: string
        of submission: submission: string
        of toast: toast: string
    Console* = object
        inputBuf: array[256, char]
        history: Deque[HistoryItem]
        inputHistory: Deque[string]

proc addToHistory(self: var Console, item: sink HistoryItem)
proc inputTextCallback(data: ptr ImGuiInputTextCallbackData): cint {.noconv.}
proc submit(self: var Console)

proc draw*(self: var Console, left: bool, menuBarHeight: float32) =
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
        clipper.begin(self.history.len.cint)

        while clipper.step():
            for i in clipper.displayStart ..< clipper.displayEnd:
                case self.history[i].discrim:
                of HistoryKind.log:
                    imGuiTextUnformatted(self.history[i].log.cStr())
                of HistoryKind.submission:
                    imGuiTextUnformatted(self.history[i].submission.cStr())
                of HistoryKind.toast:
                    imGuiTextUnformatted(self.history[i].toast.cStr())

    if imGuiInputText(
        cstring"##console.inputBuf",
        self.inputBuf[0].addr,
        self.inputBuf.len.csize_t,
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


proc log*(self: var Console, msg: string) =
    echo(msg)
    self.addToHistory(HistoryItem(discrim: HistoryKind.log, log: msg))


proc addToast*(self: var Console, msg: string) =
    self.addToHistory(HistoryItem(discrim: HistoryKind.toast, toast: msg))


# Internal #####################################################################

proc addToHistory(self: var Console, item: sink HistoryItem) =
    if self.history.len > 1024:
        self.history.popFirst()

    self.history.addLast(item)


proc inputTextCallback(data: ptr ImGuiInputTextCallbackData): cint {.noconv.} =
    echo($data.eventFlag)
    return 0


proc submit(self: var Console) =
    let submission = $cast[cstring](self.inputBuf[0].addr)

    if self.inputHistory.len > 256:
        self.inputHistory.popFirst()

    self.inputHistory.addLast(submission)
    let logged = "$ " & submission
    echo(logged)
    self.addToHistory(HistoryItem(discrim: HistoryKind.submission, submission: logged))
    self.inputBuf = default(typeof(self.inputBuf))
