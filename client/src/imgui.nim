## Compiles ImGui 1.90.5 (231cbee) and wraps its C++ API.

import stdx

const includePathFlags = staticExec("pkg-config --cflags-only-I sdl2")

{.compile("../../depend/imgui/imgui_demo.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui_draw.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui_impl_opengl3.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui_impl_sdl2.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui_tables.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui_widgets.cpp", includePathFlags).}
{.compile("../../depend/imgui/imgui.cpp", includePathFlags).}

const hImGui = "<imgui.h>"
const hImGuiImplOpenGl3 = "<imgui_impl_opengl3.h>"
const hImGuiImplSdl2 = "<imgui_impl_sdl2.h>"

{.emit: """
/*INCLUDESECTION*/
#define IMGUI_DISABLE_OBSOLETE_FUNCTIONS
#define IMGUI_DISABLE_OBSOLETE_KEYIO
""".}

type
    ImWchar32* = cuint
    ImWchar16* = cushort
    ImWchar* = ImWchar16

type ImGuiCol* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiCol_",
    header: hImGui,
.} = enum
    text
    textDisabled
    windowBg
    childBg
    popupBg
    border
    borderShadow
    frameBg
    frameBgHovered
    frameBgActive
    titleBg
    titleBgActive
    titleBgCollapsed
    menuBarBg
    scrollbarBg
    scrollbarGrab
    scrollbarGrabHovered
    scrollbarGrabActive
    checkMark
    sliderGrab
    sliderGrabActive
    button
    buttonHovered
    buttonActive
    header
    headerHovered
    headerActive
    separator
    separatorHovered
    separatorActive
    resizeGrip
    resizeGripHovered
    resizeGripActive
    tab
    tabHovered
    tabActive
    tabUnfocused
    tabUnfocusedActive
    plotLines
    plotLinesHovered
    plotHistogram
    plotHistogramHovered
    tableHeaderBg
    tableBorderStrong
    tableBorderLight
    tableRowBg
    tableRowBgAlt
    textSelectedBg
    dragDropTarget
    navHighlight
    navWindowingHighlight
    navWindowingDimBg
    modalWindowDimBg

type ImGuiCond* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiCond_",
    header: hImGui,
.} = enum
    none = 0
    always = 1 shl 0
    once = 1 shl 1
    firstUseEver = 1 shl 2
    appearing = 1 shl 3
bitFlags(ImGuiCond, cint)

type ImGuiChildFlags* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiChildFlags",
    header: hImGui,
.} = enum
    none = 0
    border = 1 shl 0
    alwaysUseWindowPadding = 1 shl 1
    resizeX = 1 shl 2
    resizeY = 1 shl 3
    autoResizeX = 1 shl 4
    autoResizeY = 1 shl 5
    alwaysAutoResize = 1 shl 6
    frameStyle = 1 shl 7
bitFlags(ImGuiChildFlags, cint)

type ImGuiConfigFlags* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiConfigFlags",
    header: hImGui,
.} = enum
    navEnableKeyboard = 1 shl 0
    navEnableGamepad = 1 shl 1
    navEnableSetMousePos = 1 shl 2
    navNoCaptureKeyboard = 1 shl 3
    noMouse = 1 shl 4
    noMouseCursorChange = 1 shl 5
bitFlags(ImGuiConfigFlags, cint)

type ImGuiInputTextFlags* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiInputTextFlags",
    header: hImGui,
.} = enum
    none = 0
    charsDecimal = 1 shl 0
    charsHexadecimal = 1 shl 1
    charsUppercase = 1 shl 2
    charsNoBlank = 1 shl 3
    autoSelectAll = 1 shl 4
    enterReturnsTrue = 1 shl 5
    callbackCompletion = 1 shl 6
    callbackHistory = 1 shl 7
    callbackAlways = 1 shl 8
    callbackCharFilter = 1 shl 9
    allowTabInput = 1 shl 10
    ctrlEnterForNewLine = 1 shl 11
    noHorizontalScroll = 1 shl 12
    alwaysOverwrite = 1 shl 13
    readOnly = 1 shl 14
    password = 1 shl 15
    noUndoRedo = 1 shl 16
    charsScientific = 1 shl 17
    callbackResize = 1 shl 18
    callbackEdit = 1 shl 19
    escapeClearsAll = 1 shl 20
bitFlags(ImGuiInputTextFlags, cint)

type ImGuiKey* {.
    size: sizeof(cint),
    importcpp: "ImGuiKey",
    header: hImGui,
.} = enum
    imkeyNone = 0
    imkeyTab = 512
    imkeyLeftArrow
    imkeyRightArrow
    imkeyUpArrow
    imkeyDownArrow
    imkeyPageUp
    imkeyPageDown
    imkeyHome
    imkeyEnd
    imkeyInsert
    imkeyDelete
    imkeyBackspace
    imkeySpace
    imkeyEnter
    imkeyEscape
    imkeyLeftCtrl, imkeyLeftShift, imkeyLeftAlt, imkeyLeftSuper
    imkeyRightCtrl, imkeyRightShift, imkeyRightAlt, imkeyRightSuper
    imkeyMenu
    imkey0, imkey1, imkey2, imkey3, imkey4, imkey5, imkey6, imkey7, imkey8, imkey9,
    imkeyA, imkeyB, imkeyC, imkeyD, imkeyE, imkeyF, imkeyG, imkeyH, imkeyI, imkeyJ
    imkeyK, imkeyL, imkeyM, imkeyN, imkeyO, imkeyP, imkeyQ, imkeyR, imkeyS, imkeyT
    imkeyU, imkeyV, imkeyW, imkeyX, imkeyY, imkeyZ
    imkeyF1, imkeyF2, imkeyF3, imkeyF4, imkeyF5, imkeyF6
    imkeyF7, imkeyF8, imkeyF9, imkeyF10, imkeyF11, imkeyF12
    imkeyF13, imkeyF14, imkeyF15, imkeyF16, imkeyF17, imkeyF18
    imkeyF19, imkeyF20, imkeyF21, imkeyF22, imkeyF23, imkeyF24

type ImGuiWindowFlags* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiWindowFlags",
    header: hImGui,
.} = enum
    none = 0
    noTitleBar = 1 shl 0
    noResize = 1 shl 1
    noMove = 1 shl 2
    noScrollbar = 1 shl 3
    noScrollWithMouse = 1 shl 4
    noCollapse = 1 shl 5
    alwaysAutoResize = 1 shl 6
    noBackground = 1 shl 7
    noSavedSettings = 1 shl 8
    noMouseInputs = 1 shl 9
    menuBar = 1 shl 10
    horizontalScrollbar = 1 shl 11
    noFocusOnAppearing = 1 shl 12
    noBringToFrontOnFocus = 1 shl 13
    alwaysVerticalScrollbar = 1 shl 14
    alwaysHorizontalScrollbar = 1 shl 15
    noNavInputs = 1 shl 16
    noNavFocus = 1 shl 17
    unsavedDocument = 1 shl 18
    noNav = 1 shl 19
    noDecoration = 1 shl 20
    noInputs = 1 shl 21
bitFlags(ImGuiWindowFlags, cint)

type ImGuiComboFlags* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiComboFlags",
    header: hImGui,
.} = enum
    none = 0
    popupAlignLeft = 1 shl 0
    heightSmall = 1 shl 1
    heightRegular = 1 shl 2
    heightLarge = 1 shl 3
    heightLargest = 1 shl 4
    noArrowButton = 1 shl 5
    noPreview = 1 shl 6
    widthFitPreview = 1 shl 7
bitFlags(ImGuiComboFlags, cint)

type ImVec2* {.importcpp: "ImVec2", header: hImGui.} = object
    x*, y*: float32 = 0.0.float32

type ImVec4* {.importcpp: "ImVec4", header: hImGui.} = object
    x*, y*, z*, w*: float32 = 0.0.float32

type ImGuiContext* {.
    importcpp: "ImGuiContext",
    header: hImGui,
    incompleteStruct,
.} = object

type ImGuiInputTextCallbackData* {.
    importcpp: "ImGuiInputTextCallbackData",
    header: hImGui,
    incompleteStruct,
.} = object
    ctx* {.importcpp: "Ctx".}: ptr ImGuiContext
    eventFlag* {.importcpp: "EventFlag".}: ImGuiInputTextFlags
    flags* {.importcpp: "Flags".}: ImGuiInputTextFlags
    userData* {.importcpp: "UserData".}: pointer
    eventChar* {.importcpp: "EventChar".}: ImWchar
    eventKey* {.importcpp: "EventKey".}: ImGuiKey
    buf* {.importcpp: "Buf".}: cstring
    bufTextLen* {.importcpp: "BufTextLen".}: cint
    bufSize* {.importcpp: "BufSize".}: cint
    bufDirty* {.importcpp: "BufDirty".}: bool
    cursorPos* {.importcpp: "CursorPos".}: cint
    selectionStart* {.importcpp: "SelectionStart".}: cint
    selectionEnd* {.importcpp: "SelectionEnd".}: cint

type ImGuiInputTextCallback* {.
    importcpp: "ImGuiInputTextCallback",
    header: hImGui,
.} = proc(data: ptr ImGuiInputTextCallbackData): cint {.noconv.}

type ImGuiIO* {.
    importcpp: "ImGuiIO",
    header: hImGui,
    incompleteStruct,
.} = object
    configFlags* {.importcpp: "ConfigFlags".}: ImGuiConfigFlags
    displayFramebufferScale* {.importcpp: "DisplayFramebufferScale".}: ImVec2
    wantCaptureMouse* {.importcpp: "WantCaptureMouse".}: bool
    wantCaptureKeyboard* {.importcpp: "WantCaptureKeyboard".}: bool
    wantTextInput* {.importcpp: "WantTextInput".}: bool

type ImGuiListClipper* {.
    importcpp: "ImGuiListClipper",
    header: hImGui,
    incompleteStruct,
.} = object
    displayStart* {.importcpp: "DisplayStart".}: cint
    displayEnd* {.importcpp: "DisplayEnd".}: cint
    itemsCount* {.importcpp: "ItemsCount".}: cint
    itemsHeight* {.importcpp: "ItemsHeight".}: float32
    startPosY* {.importcpp: "StartPosY".}: float32

proc init*(_: typedesc[ImGuiListClipper]): ImGuiListClipper
    {.importcpp: "ImGuiListClipper(@)", constructor, header: hImGui.}

proc begin*(self: var ImGuiListClipper, itemsCount: cint, itemsHeight: float32 = -1.0)
    {.importcpp: "#.Begin(@)", header: hImGui.}

proc `end`*(self: var ImGuiListClipper)
    {.importcpp: "#.End(@)", header: hImGui.}

proc step*(self: var ImGuiListClipper): bool
    {.importcpp: "#.Step(@)", header: hImGui.}

type ImGuiStyle* {.
    importcpp: "ImGuiStyle",
    header: hImGui,
    incompleteStruct,
.} = object
    alpha* {.importcpp: "Alpha".}: float32
    disabledAlpha* {.importcpp: "DisabledAlpha".}: float32
    itemSpacing* {.importcpp: "ItemSpacing".}: ImVec2
    itemInnerSpacing* {.importcpp: "ItemInnerSpacing".}: ImVec2

type ImGuiViewport* {.
    importcpp: "ImGuiViewport",
    header: hImGui,
    incompleteStruct,
.} = object
    pos* {.importcpp: "Pos".}: ImVec2
    size* {.importcpp: "Size".}: ImVec2
    workPos* {.importcpp: "WorkPos".}: ImVec2
    workSize* {.importcpp: "WorkSize".}: ImVec2
    platformHandleRaw {.importcpp: "PlatformHandleRaw".}: pointer

proc getCenter*(self: ImGuiViewport): ImVec2
    {.importcpp: "#.GetCenter(@)", header: hImGui.}

proc getWorkCenter*(self: ImGuiViewport): ImVec2
    {.importcpp: "#.GetWorkCenter(@)", header: hImGui.}

type ImDrawData* {.
    importcpp: "ImDrawData",
    header: hImGui,
    incompleteStruct,
.} = object

type ImFontAtlas* {.
    importcpp: "ImFontAtlas",
    header: hImGui,
    incompleteStruct,
.} = object

type SdlEvent* {.
    importc: "SDL_Event",
    header: hImGuiImplSdl2,
    union,
    incompleteStruct,
.} = object

type SdlWindow* {.
    importc: "SDL_Window",
    header: hImGuiImplSdl2,
    incompleteStruct,
.} = object

# Context creation and access ##################################################

proc createImGuiContext*(sharedFontAtlas: ptr ImFontAtlas): ptr ImGuiContext
    {.importcpp: "ImGui::CreateContext(@)", header: hImGui.}

proc currentImGuiContext*(): ptr ImGuiContext
    {.importcpp: "ImGui::GetCurrentContext(@)", header: hImGui.}

proc destroyImGuiContext*(ctx: ptr ImGuiContext = nil)
    {.importcpp: "ImGui::DestroyContext(@)", header: hImGui.}

proc setImGuiContext*(ctx: ptr ImGuiContext)
    {.importcpp: "ImGui::SetCurrentContext(@)", header: hImGui.}

# Main #########################################################################

proc get*(_: typedesc[ImGuiIO]): var ImGuiIO
    {.importcpp: "ImGui::GetIO(@)", header: hImGui.}

proc get*(_: typedesc[ImGuiStyle]): var ImGuiStyle
    {.importcpp: "ImGui::GetStyle(@)", header: hImGui.}

proc render*(self: ptr ImDrawData)
    {.importcpp: "ImGui_ImplOpenGL3_RenderDrawData(@)", header: hImGuiImplOpenGl3.}

proc imGuiNewFrame*()
    {.importcpp: "ImGui::NewFrame(@)", header: hImGui.}

proc imGuiEndFrame*()
    {.importcpp: "ImGui::EndFrame(@)", header: hImGui.}

proc imGuiRender*()
    {.importcpp: "ImGui::Render(@)", header: hImGui.}

proc get*(_: typedesc[ImDrawData]): ptr ImDrawData
    {.importcpp: "ImGui::GetDrawData(@)", header: hImGui.}

# Demo, debug, information #####################################################

proc imGuiShowDemoWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowDemoWindow(@)", header: hImGui.}

proc imGuiShowMetricsWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowMetricsWindow(@)", header: hImGui.}

proc imGuiShowDebugLogWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowDebugLogWindow(@)", header: hImGui.}

proc imGuiShowIdStackToolWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowIDStackToolWindow(@)", header: hImGui.}

proc imGuiShowAboutWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowAboutWindow(@)", header: hImGui.}

proc imGuiShowStyleEditor*(reference: ptr ImGuiStyle = nil)
    {.importcpp: "ImGui::ShowStyleEditor(@)", header: hImGui.}

proc imGuiShowStyleSelector*(label: cstring): bool
    {.importcpp: "ImGui::ShowStyleSelector(@)", header: hImGui.}

proc imGuiShowFontSelector*(label: cstring)
    {.importcpp: "ImGui::ShowFontSelector(@)", header: hImGui.}

proc imGuiShowUserGuide*()
    {.importcpp: "ImGui::ShowUserGuide(@)", header: hImGui.}

proc imGuiVersion*(): cstring
    {.importcpp: "ImGui::GetVersion(@)", header: hImGui.}

# Styles #######################################################################

proc imGuiStyleColorsDark*(dst: ptr ImGuiStyle = nil)
    {.importcpp: "ImGui::StyleColorsDark(@)", header: hImGui.}

# Windows ######################################################################

proc imGuiBeginWindow*(
    name: cstring,
    pOpen: ptr bool = nil,
    flags: ImGuiWindowFlags = ImGuiWindowFlags.none
): bool
    {.importcpp: "ImGui::Begin(@)", header: hImGui.}

proc imGuiEndWindow*()
    {.importcpp: "ImGui::End(@)", header: hImGui.}

# Child windows ################################################################

proc imGuiBeginChild*(
    strId: cstring,
    size {.byref.}: ImVec2 = ImVec2(x: 0.0, y: 0.0),
    childFlags: ImGuiChildFlags = ImGuiChildFlags.none,
    windowFlags: ImGuiWindowFlags = ImGuiWindowFlags.none,
): bool
    {.importcpp: "ImGui::BeginChild(@)", header: hImGui.}

proc imGuiEndChild*()
    {.importcpp: "ImGui::EndChild(@)", header: hImGui}

# Window utilities #############################################################

proc imGuiGetWindowPos*(): ImVec2
    {.importcpp: "ImGui::GetWindowPos(@)", header: hImGui}

proc imGuiGetWindowSize*(): ImVec2
    {.importcpp: "ImGui::GetWindowSize(@)", header: hImGui}

proc imGuiGetWindowWidth*(): float32
    {.importcpp: "ImGui::GetWindowWidth(@)", header: hImGui}

proc imGuiGetWindowHeight*(): float32
    {.importcpp: "ImGui::GetWindowHeight(@)", header: hImGui}

# Window manipulation ##########################################################

proc imGuiSetNextWindowPos*(
    pos {.byref.}: ImVec2,
    cond: ImGuiCond = ImGuiCond.none,
    pivot {.byref.}: ImVec2 = ImVec2(x: 0.0, y: 0.0),
)
    {.importcpp: "ImGui::SetNextWindowPos(@)", header: hImGui.}

proc imGuiSetNextWindowSize*(
    size {.byref.}: ImVec2,
    cond: ImGuiCond = ImGuiCond.none
)
    {.importcpp: "ImGui::SetNextWindowSize(@)", header: hImGui.}

# Parameter stacks (shared) ####################################################

proc imGuiPushStyleColor*(idx: ImGuiCol, col: cuint)
    {.importcpp: "ImGui::PushStyleColor(@)", header: hImGui.}

proc imGuiPushStyleColor*(idx: ImGuiCol, col {.byref.}: ImVec4)
    {.importcpp: "ImGui::PushStyleColor(@)", header: hImGui.}

proc imGuiPopStyleColor*(count: cint = 1)
    {.importcpp: "ImGui::PopStyleColor(@)", header: hImGui.}

# Parameter stacks (current window) ############################################

proc imGuiPushItemWidth*(width: float32)
    {.importcpp: "ImGui::PushItemWidth(@)", header: hImGui.}

proc imGuiPopItemWidth*()
    {.importcpp: "ImGui::PopItemWidth(@)", header: hImGui.}

proc imGuiSetNextItemWidth*(width: float32)
    {.importcpp: "ImGui::SetNextItemWidth(@)", header: hImGui}

proc imGuiCalcItemWidth*(): float32
    {.importcpp: "ImGui::CalcItemWidth(@)", header: hImGui.}

proc imGuiPushTextWrapPos*(wrapLocalPosX: float32)
    {.importcpp: "ImGui::PushTextWrapPos(@)", header: hImGui.}

proc imGuiPopTextWrapPos*()
    {.importcpp: "ImGui::PopTextWrapPos(@)", header: hImGui.}

# Other layout functions #######################################################

proc imGuiSeparator*()
    {.importcpp: "ImGui::Separator(@)", header: hImGui.}

proc imGuiSameLine*(offsetFromStartX: float32 = 0.0, spacing: float32 = -1.0)
    {.importcpp: "ImGui::SameLine(@)", header: hImGui.}

proc imGuiNewLine*()
    {.importcpp: "ImGui::NewLine(@)", header: hImGui.}

proc imGuiSpacing*()
    {.importcpp: "ImGui::Spacing(@)", header: hImGui.}

proc imGuiDummy*(size: var ImVec2)
    {.importcpp: "ImGui::Dummy(@)", header: hImGui.}

proc imGuiIndent*(indentW: float32 = 0.0)
    {.importcpp: "ImGui::Indent(@)", header: hImGui.}

proc imGuiUnindent*(indentW: float32 = 0.0)
    {.importcpp: "ImGui::Unindent(@)", header: hImGui.}

proc imGuiBeginGroup*()
    {.importcpp: "ImGui::BeginGroup(@)", header: hImGui.}

proc imGuiEndGroup*()
    {.importcpp: "ImGui::EndGroup(@)", header: hImGui.}

proc imGuiAlignTextToFramePadding*()
    {.importcpp: "ImGui::AlignTextToFramePadding(@)", header: hImGui.}

proc imGuiGetTextLineHeight*(): float32
    {.importcpp: "ImGui::GetTextLineHeight(@)", header: hImGui.}

proc imGuiGetTextLineHeightWithSpacing*(): float32
    {.importcpp: "ImGui::GetTextLineHeightWithSpacing(@)", header: hImGui.}

proc imGuiGetFrameHeight*(): float32
    {.importcpp: "ImGui::GetFrameHeight(@)", header: hImGui.}

proc imGuiGetFrameHeightWithSpacing*(): float32
    {.importcpp: "ImGui::GetFrameHeightWithSpacing(@)", header: hImGui.}

# Widgets: text ################################################################

proc imGuiTextUnformatted*(text: cstring, textEnd: cstring = nil)
    {.importcpp: "ImGui::TextUnformatted(@)", header: hImGui.}

proc imGuiText*(fmt: cstring)
    {.importcpp: "ImGui::Text(@)", header: hImGui, varargs.}

# Widgets: main ################################################################

proc imGuiButton*(label: cstring, size: ImVec2 = ImVec2()): bool
    {.importcpp: "ImGui::Button(@)", header: hImGui.}

# Widgets: combo box (dropdown) ################################################

proc imGuiBeginCombo*(
    label, previewVal: cstring,
    flags: ImGuiComboFlags = ImGuiComboFlags.none
): bool
    {.importcpp: "ImGui::BeginCombo(@)", header: hImGui.}

proc imGuiEndCombo*()
    {.importcpp: "ImGui::EndCombo(@)", header: hImGui.}

proc imGuiCombo*(
    label: cstring,
    currentItem: ptr cint,
    items: ptr cstring,
    numItems: cint,
    popupMaxHeightInItems: cint = -1,
): bool
    {.importcpp: "ImGui::Combo(@)", header: hImgui.}

# Widgets: input with keyboard #################################################

proc imGuiInputText*(
    label: cstring,
    buf: ptr char,
    bufSize: csize_t,
    flags: ImGuiInputTextFlags = ImGuiInputTextFlags.none,
    callback: ImGuiInputTextCallback = nil,
    userData: pointer = nil,
): bool
    {.importcpp: "ImGui::InputText(@)", header: hImGui.}

# Widgets: menus ###############################################################

proc imGuiBeginMenuBar*(): bool
    {.importcpp: "ImGui::BeginMenuBar(@)", header: hImGui.}

proc imGuiEndMenuBar*()
    {.importcpp: "ImGui::EndMenuBar(@)", header: hImGui.}

proc imGuiBeginMainMenuBar*(): bool
    {.importcpp: "ImGui::BeginMainMenuBar(@)", header: hImGui.}

proc imGuiEndMainMenuBar*()
    {.importcpp: "ImGui::EndMainMenuBar(@)", header: hImGui.}

proc imGuiBeginMenu*(label: cstring, enabled: bool = true): bool
    {.importcpp: "ImGui::BeginMenu(@)", header: hImGui.}

proc imGuiEndMenu*()
    {.importcpp: "ImGui::EndMenu(@)", header: hImGui.}

proc imGuiMenuItem*(
    label: cstring,
    shortcut: cstring = nil,
    selected: bool = false,
    enabled: bool = true,
): bool
    {.importcpp: "ImGui::MenuItem(@)", header: hImGui.}

proc imGuiMenuItem*(
    label: cstring,
    shortcut: cstring,
    pSelected: ptr bool,
    enabled: bool = true
): bool
    {.importcpp: "ImGui::MenuItem(@)", header: hImGui.}

# Tooltips #####################################################################

proc imGuiBeginTooltip*(): bool
    {.importcpp: "ImGui::BeginTooltip(@)", header: hImGui.}

proc imGuiEndTooltip*()
    {.importcpp: "ImGui::BeginTooltip(@)", header: hImGui.}

# Tooltips: helpers for showing a tooltip when hovering an item ################

proc imGuiBeginItemTooltip*(): bool
    {.importcpp: "ImGui::BeginItemTooltip(@)", header: hImGui.}

# Focus, activation ############################################################

proc imGuiSetItemDefaultFocus*()
    {.importcpp: "ImGui::SetItemDefaultFocus(@)", header: hImGui.}

# Viewports ####################################################################

proc imGuiGetMainViewport*(): ptr ImGuiViewport
    {.importcpp: "ImGui::GetMainViewport", header: hImGui.}

# OpenGL3 backend ##############################################################

proc imGuiOpenGl3Setup*(glslVersion: cstring = nil): bool
    {.importcpp: "ImGui_ImplOpenGL3_Init(@)", header: hImGuiImplOpenGl3.}

proc imGuiOpenGl3NewFrame*()
    {.importcpp: "ImGui_ImplOpenGL3_NewFrame(@)", header: hImGuiImplOpenGl3.}

proc imGuiOpenGl3Shutdown*()
    {.importcpp: "ImGui_ImplOpenGL3_Shutdown", header: hImGuiImplOpenGl3.}

# SDL2 backend #################################################################

proc imGuiSdl2NewFrame*()
    {.importcpp: "ImGui_ImplSDL2_NewFrame(@)", header: hImGuiImplSdl2.}

proc imGuiSdl2OpenGlSetup*(window: ptr SdlWindow, sdlGlContext: pointer): bool
    {.importcpp: "ImGui_ImplSDL2_InitForOpenGL(@)", header: hImGuiImplSdl2.}

proc imGuiSdl2Shutdown*()
    {.importcpp: "ImGui_ImplSDL2_Shutdown(@)", header: hImGuiImplSdl2.}

proc process*(self: ptr SdlEvent): bool
    {.importcpp: "ImGui_ImplSDL2_ProcessEvent(@)", header: hImGuiImplSdl2.}
