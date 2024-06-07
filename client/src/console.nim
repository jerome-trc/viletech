import imgui

type
    HistoryKind {.pure.} = enum
        toast
        submission
    HistoryItem = object
        case discrim: HistoryKind
        of toast: toast: string
        of submission: submission: string
    Console* = object
        inputBuf: array[256, char]
        history: seq[HistoryItem]

proc draw*(self: var Console, menuBarHeight: float32) =
    let vp = imguiGetMainViewport()
    imGuiSetNextWindowPos(ImVec2(y: menuBarHeight))
    imGuiSetNextWindowSize(ImVec2(x: vp.size.x * 0.5, y: vp.size.y * 0.33))

    if not imGuiBeginWindow(
        cstring"Console",
        flags = ImGuiWindowFlags.noTitleBar + ImGuiWindowFlags.noResize
    ):
        return

    defer: imGuiEndWindow()

    if imGuiInputText(
        cstring"##inputBuf",
        self.inputBuf[0].addr,
        self.inputBuf.len.csize_t,
    ):
        discard

    imGuiSameLine()

    if imGuiButton(cstring"Submit"):
        let submission = "$ " & $cast[cstring](self.inputBuf[0].addr)
        echo(submission)
        self.history.add(HistoryItem(discrim: HistoryKind.submission, submission: submission))
        self.inputBuf = default(typeof(self.inputBuf))


proc addToast*(self: var Console, msg: cstring) =
    self.history.add(HistoryItem(discrim: HistoryKind.toast, toast: $msg))
