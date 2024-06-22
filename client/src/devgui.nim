## Abstractions over ImGui.

import core, devgui/[console, vfs], imgui

proc dguiSetup*(
    self: var CCore,
    window: ptr SdlWindow,
    sdlGlCtx: pointer,
) {.exportc: "vt_$1".} =
    self.core.dgui.imguiCtx = createImGuiContext(nil)

    var io = ImGuiIO.get()
    io.configFlags += ImGuiConfigFlags.navEnableKeyboard

    discard imGuiSdl2OpenGlSetup(window, sdlGlCtx)
    discard imGuiOpenGl3Setup(nil)
    imGuiStyleColorsDark(nil)


proc dguiShutdown*() {.exportc: "vt_$1".} =
    imGuiOpenGl3Shutdown()
    imGuiSdl2Shutdown()
    destroyImGuiContext()


proc dguiFrameBegin*(self: var CCore) {.exportc: "vt_$1".} =
    imGuiOpenGl3NewFrame()
    imGuiSdl2NewFrame()
    imGuiNewFrame()


proc dguiDraw*(self: var CCore) {.exportc: "vt_$1".} =
    if not self.core.dgui.open:
        return

    if not imGuiBeginMainMenuBar():
        return

    block:
        imGuiPushStyleColor(ImGuiCol.menuBarBg, ImVec4(w: 0.66))
        imGuiPushStyleColor(ImGuiCol.windowBg, ImVec4(w: 0.66))

        defer: imGuiPopStyleColor(2); imGuiEndMainMenuBar()

        imGuiTextUnformatted("Developer Tools")
        imGuiSeparator()

        if imGuiMenuItem(cstring"Close"):
            self.core.dgui.open = false

        if imGuiMenuItem(cstring"ImGui Metrics"):
            self.core.dgui.metricsWindow = not self.core.dgui.metricsWindow

        let vp = imGuiGetMainViewport()

        let items = [
            cstring"Console",
            cstring"Nim Playground",
            cstring"VFS",
        ]

        imGuiPushItemWidth(vp.size.x * 0.15)

        if imGuiCombo(
            cstring"Left",
            cast[ptr cint](self.core.dgui.left.addr),
            cast[ptr cstring](items.addr),
            items.len.cint
        ):
            # ImGui misbehaves if both sides of the developer GUI draw the same tool.
            if self.core.dgui.left == self.core.dgui.right:
                for i in DevGui.items:
                    if self.core.dgui.left != i:
                        self.core.dgui.right = i

        if imGuiCombo(
            cstring"Right",
            cast[ptr cint](self.core.dgui.right.addr),
            cast[ptr cstring](items.addr),
            items.len.cint
        ):
            if self.core.dgui.left == self.core.dgui.right:
                for i in DevGui.items:
                    if self.core.dgui.right != i:
                        self.core.dgui.left = i

        imGuiPopItemWidth()

        let menuBarHeight = imGuiGetWindowHeight()

        case self.core.dgui.left:
        of DevGui.console: console.draw(self.core[], left = true, menuBarHeight)
        of DevGui.playground: discard
        of DevGui.vfs: vfs.draw(self.core[], false, menuBarHeight)

        case self.core.dgui.right:
        of DevGui.console: console.draw(self.core[], left = false, menuBarHeight)
        of DevGui.playground: discard
        of DevGui.vfs: vfs.draw(self.core[], false, menuBarHeight)

    if self.core.dgui.metricsWindow:
        imGuiShowMetricsWindow(nil)


proc dguiFrameFinish*(self: var CCore) {.exportc: "vt_$1".} =
    imGuiRender()


proc dguiFrameDraw*(self: var CCore) {.exportc: "vt_$1".} =
    ImDrawData.get().render()


proc dguiToggle*(self: var CCore): bool {.exportc: "vt_$1".} =
    ## Returns `true` if the developer GUI is open after the toggle.
    self.core.dgui.open = not self.core.dgui.open
    return self.core.dgui.open


proc dguiIsOpen*(self: var CCore): bool {.exportc: "vt_$1".} =
    self.core.dgui.open


proc dguiWantsKeyboard*(self: var CCore): bool {.exportc: "vt_$1".} =
    ImGuiIO.get().wantCaptureKeyboard


proc dguiWantsMouse*(self: var CCore): bool {.exportc: "vt_$1".} =
    ImGuiIO.get().wantCaptureMouse


proc processEvent*(self: var CCore, event: ptr SdlEvent): bool {.exportc: "vt_$1".} =
    return event.process()
