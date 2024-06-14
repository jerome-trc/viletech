import std/[options, strformat]

import ../[core, imgui, imports, stdx]
import console

proc contextMenu(self: var Core, num: LumpNum)


proc draw*(self: var Core, left: bool, menuBarHeight: float32) =
    let vp = imGuiGetMainViewport()

    if left:
        imGuiSetNextWindowPos(ImVec2(y: menuBarHeight))
    else:
        imGuiSetNextWindowPos(ImVec2(x: vp.size.x * 0.5, y: menuBarHeight))

    imGuiSetNextWindowSize(ImVec2(x: vp.size.x * 0.5, y: vp.size.y * 0.33))

    if not imGuiBeginWindow(
        cstring"Virtual File System",
        flags =
            ImGuiWindowFlags.noTitleBar +
            ImGuiWindowFlags.noResize +
            ImGuiWindowFlags.menuBar
    ):
        return

    defer: imGuiEndWindow()

    if imGuiBeginMenuBar():
        defer: imGuiEndMenuBar()
        # TODO: regex-based name filtration.

    if imGuiBeginTable("##vfsTable", numColumns = 3, flags = ImGuiTableFlags.bordersH):
        defer: imGuiEndTable()

        imGuiTableSetupColumn(
            cstring"#",
            ImGuiTableColumnFlags.widthFixed,
            initWidthOrHeight = vp.size.x * 0.05,
        )
        imGuiTableSetupColumn(cstring"Name")
        imGuiTableSetupColumn(cstring"Size")
        imGuiTableHeadersRow()

        var clipper = ImGuiListClipper.init()
        clipper.begin(numLumps.cint)
        var popupShown = false

        while clipper.step():
            for i in clipper.displayStart ..< clipper.displayEnd:
                imGuiTableNextRow()

                if imGuiTableNextColumn():
                    imGuiText(cstring"%d", i)

                if imGuiBeginPopupContextItem(cstring"##vfsContext"):
                    defer: imGuiEndPopup()
                    if not popupShown: self.contextMenu(i.LumpNum)
                    popupShown = true

                if imGuiTableNextColumn():
                    imGuiTextUnformatted(lumpName(i.LumpNum))

                if imGuiBeginPopupContextItem(cstring"##vfsContext"):
                    defer: imGuiEndPopup()
                    if not popupShown: self.contextMenu(i.LumpNum)
                    popupShown = true

                if imGuiTableNextColumn():
                    let txt = subdivideBytes(lumpLength(i.LumpNum))
                    imGuiTextUnformatted(txt.cStr)

                if imGuiBeginPopupContextItem(cstring"##vfsContext"):
                    defer: imGuiEndPopup()
                    if not popupShown: self.contextMenu(i.LumpNum)
                    popupShown = true


proc asciiId*(a, b, c, d: char): uint32 =
    let a32 = a.uint32
    let b32 = b.uint32
    let c32 = c.uint32
    let d32 = d.uint32

    when cpuEndian == bigEndian:
        return a32 or (b32 shl 8) or (c32 shl 16) or (d32 shl 24)
    else:
        return d32 or (c32 shl 8) or (b32 shl 16) or (a32 shl 24)


proc contextMenu(self: var Core, num: LumpNum) =
    let lump = try: getLump(num).get()
    except: return

    if lump.len <= 0: return

    if imGuiButton(cstring"Copy to Clipboard"):
        self.dgui.console.log(&"Copied {lump.name}'s contents to the clipboard.")

    if lump.len < 4: return

    try:
        let magic4 = [lump.bytes[0], lump.bytes[1], lump.bytes[2], lump.bytes[3]]
        let magic = cast[uint32](magic4)

        var isMusic = false
        isMusic = isMusic or magic == asciiId('M', 'T', 'h', 'd')
        # TODO: MUS and raw formats.

        if isMusic and imGuiButton(cstring"Play Music"):
            discard changeMusicByName(self.c.addr, lump.name, true)
    except Exception as e:
        assert(false, e.msg)
