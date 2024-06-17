import std/[options, re, strformat]

import ../[core, imgui, imports, stdx]
import console

proc tryContextMenu(self: var Core, popupShown: var bool, lump {.byref.}: Lump)


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

        if imGuiInputText(
            cstring"Filter",
            self.dgui.vfs.filterBuf[0].addr,
            self.dgui.vfs.filterBuf.len.csize_t,
        ):
            self.dgui.vfs.filter = re(
                self.dgui.vfs.filterBuf.substr(),
                {reIgnoreCase, reStudy}
            ) # Garbage factory...

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
            var i = clipper.displayStart
            var l = i

            while (i < clipper.displayEnd) and (l < numLumps.cint):
                defer: l += 1

                let o = getLump(l.LumpNum)
                if o.isNone: continue
                let lump = o.unsafeGet()

                var matches: array[0, string] = []

                if not lump.name.match(self.dgui.vfs.filter, matches, bufSize = 8):
                    continue

                imGuiTableNextRow()
                defer: i += 1

                if imGuiTableNextColumn():
                    imGuiText(cstring"%d", l)

                self.tryContextMenu(popupShown, lump)

                if imGuiTableNextColumn():
                    imGuiTextUnformatted(lump.name)

                self.tryContextMenu(popupShown, lump)

                if imGuiTableNextColumn():
                    let txt = subdivideBytes(lumpLength(i.LumpNum))
                    imGuiTextUnformatted(txt.cStr)

                self.tryContextMenu(popupShown, lump)


proc asciiId*(a, b, c, d: char): uint32 =
    when cpuEndian == bigEndian:
        return cast[uint32]([d, c, b, a])
    else:
        return cast[uint32]([a, b, c, d])


proc tryContextMenu(self: var Core, popupShown: var bool, lump {.byref.}: Lump) =
    if not imGuiBeginPopupContextItem("##vfs.context"): return
    defer: imGuiEndPopup()

    if popupShown or (lump.len <= 0): return
    popupShown = true

    if imGuiButton(cstring"Copy to Clipboard"):
        self.dgui.console.log(&"Copied {lump.name}'s contents to the clipboard.")

    if lump.len < 4: return

    try:
        let magic4 = [lump.bytes[0], lump.bytes[1], lump.bytes[2], lump.bytes[3]]
        let magic = cast[uint32](magic4)

        var isMusic = false
        isMusic = isMusic or (magic == asciiId('M', 'T', 'h', 'd'))
        isMusic = isMusic or (magic == asciiId('R', 'I', 'F', 'F'))
        isMusic = isMusic or (magic == asciiId('M', 'I', 'D', 'S'))
        # TODO: MUS and raw formats.

        if isMusic and imGuiButton(cstring"Play Music"):
            discard changeMusicByName(self.c.addr, lump.name, true)
    except Exception as e:
        assert(false, e.msg)
