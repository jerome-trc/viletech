## Abstractions over ImGui.

import core, imgui

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
    newImGuiFrame()


proc dguiDraw*(self: var CCore) {.exportc: "vt_$1".} =
    showImGuiMetricsWindow(nil)

    if not beginImGuiWindow(cstring"Console", self.core.dgui.consoleOpen.addr, ImGuiWindowFlags.none):
        endImGuiWindow()
        return

    if imGuiButton(cstring"Submit"):
        return

    endImGuiWindow()


proc dguiFrameFinish*(self: var CCore) {.exportc: "vt_$1".} =
    imGuiRender()


proc dguiFrameDraw*(self: var CCore) {.exportc: "vt_$1".} =
    ImDrawData.get().render()


proc dguiNeedsMouse*(self: var CCore): bool {.exportc: "vt_$1".} =
    self.core.dgui.consoleOpen


proc processEvent*(self: var CCore, event: ptr SdlEvent) {.exportc: "vt_$1".} =
    event.process()
