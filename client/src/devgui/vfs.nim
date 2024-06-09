import ../imgui

proc draw*(left: bool, menuBarHeight: float32) =
    let vp = imGuiGetMainViewport()

    if left:
        imGuiSetNextWindowPos(ImVec2(y: menuBarHeight))
    else:
        imGuiSetNextWindowPos(ImVec2(x: vp.size.x * 0.5, y: menuBarHeight))

    imGuiSetNextWindowSize(ImVec2(x: vp.size.x * 0.5, y: vp.size.y * 0.33))

    if not imGuiBeginWindow(
        cstring"Virtual File System",
        flags = ImGuiWindowFlags.noTitleBar + ImGuiWindowFlags.noResize
    ):
        return

    defer: imGuiEndWindow()
