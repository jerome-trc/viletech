## Abstractions over ImGui.

import core, imgui

proc dguiSetup*(
    this: var CCore,
    window: ptr SdlWindow,
    sdlGlCtx: pointer,
) {.exportc: "vt_$1".} =
    this.core.imguiCtx = createImGuiContext(nil)

    var io = ImGuiIO.get()
    io.configFlags.incl(ImGuiConfigFlag.navEnableKeyboard)

    discard imGuiSdl2OpenGlSetup(window, sdlGlCtx)
    discard imGuiOpenGl3Setup(nil)
    imGuiStyleColorsDark(nil)


proc dguiFrameBegin*(this: var CCore) {.exportc: "vt_$1".} =
    imGuiOpenGl3NewFrame()
    imGuiSdl2NewFrame()
    newImGuiFrame()


proc dguiDraw*(this: var CCore) {.exportc: "vt_$1".} =
    showImGuiMetricsWindow(nil)

    discard beginImGuiWindow(cstring"Console")
    endImGuiWindow()


proc dguiFrameFinish*(this: var CCore) {.exportc: "vt_$1".} =
    imGuiRender()


proc dguiFrameDraw*(this: var CCore) {.exportc: "vt_$1".} =
    ImDrawData.get().render()


proc processEvent*(this: var CCore, event: ptr SdlEvent) {.exportc: "vt_$1".} =
    event.process()
