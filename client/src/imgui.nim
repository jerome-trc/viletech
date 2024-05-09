## Compiles ImGui 1.90.5 (231cbee) and wraps its C++ API.

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

type ImGuiConfigFlag* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiConfigFlags_",
    header: hImGui,
.} = enum
    navEnableKeyboard = 1 shl 0
    navEnableGamepad = 1 shl 1
    navEnableSetMousePos = 1 shl 2
    navNoCaptureKeyboard = 1 shl 3
    noMouse = 1 shl 4
    noMouseCursorChange = 1 shl 5
type ImGuiConfigFlags* = set[ImGuiConfigFlag]

static: assert(sizeof(ImGuiConfigFlags) == 4)

type ImGuiWindowFlag* {.
    pure,
    size: sizeof(cint),
    importcpp: "ImGuiWindowFlags_",
    header: hImGui,
.} = enum
    noTitleBar = 1 shl 0
    noResize = 1 shl 1
    noMove = 1 shl 2
    noScrollbar = 1 shl 3
    noScrollWithMouse = 1 shl 4
    noCollapse = 1 shl 5
    nlwaysAutoResize = 1 shl 6
    noBackground = 1 shl 7
    noSavedSettings = 1 shl 8
    noMouseInputs = 1 shl 9
    nenuBar = 1 shl 10
    norizontalScrollbar = 1 shl 11
    noFocusOnAppearing = 1 shl 12
    noBringToFrontOnFocus = 1 shl 13
    nlwaysVerticalScrollbar = 1 shl 14
    nlwaysHorizontalScrollbar = 1 shl 15
    noNavInputs = 1 shl 16
    # TODO: a custom flag set type for these remaining bits.
    # noNavFocus = 1 shl 17
    # nnsavedDocument = 1 shl 18
    # noNav = 1 shl 19
    # noDecoration = 1 shl 20
    # noInputs = 1 shl 21
type ImGuiWindowFlags* = set[ImGuiWindowFlag]

type ImGuiContext* {.
    importcpp: "ImGuiContext",
    header: hImGui,
    incompleteStruct,
.} = object

type ImGuiIO* {.
    importcpp: "ImGuiIO",
    header: hImGui,
    incompleteStruct,
.} = object
    ConfigFlags: ImGuiConfigFlags

type ImGuiStyle* {.
    importcpp: "ImGuiStyle",
    header: hImGui,
    incompleteStruct,
.} = object

proc configFlags*(this: var ImGuiIO): var ImGuiConfigFlags {.inline.} =
    this.ConfigFlags


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
    {.importcpp: "ImGui::GetCurrentContext(@)".}

proc setImGuiContext*(ctx: ptr ImGuiContext)
    {.importcpp: "ImGui::SetCurrentContext(@)".}

# Main #########################################################################

proc get*(_: typedesc[ImDrawData]): ptr ImDrawData
    {.importcpp: "ImGui::GetDrawData(@)", header: hImGui.}

proc get*(_: typedesc[ImGuiIO]): var ImGuiIO
    {.importcpp: "ImGui::GetIO(@)", header: hImGui.}

proc render*(this: ptr ImDrawData)
    {.importcpp: "ImGui_ImplOpenGL3_RenderDrawData(@)", header: hImGuiImplOpenGl3.}

proc newImGuiFrame*()
    {.importcpp: "ImGui::NewFrame(@)", header: hImGui.}

proc endImGuiFrame*()
    {.importcpp: "ImGui::EndFrame(@)", header: hImGui.}

proc imGuiRender*()
    {.importcpp: "ImGui::Render(@)", header: hImGui.}

# Demo, debug, information #####################################################

proc getImGuiVersion*(): cstring
    {.importcpp: "ImGui::GetVersion(@)", header: hImGui.}

proc showImGuiMetricsWindow*(pOpen: ptr bool = nil)
    {.importcpp: "ImGui::ShowMetricsWindow(@)", header: hImGui.}

proc showImGuiUserGuide*()
    {.importcpp: "ImGui::ShowUserGuide(@)", header: hImGui.}

# Styles #######################################################################

proc imGuiStyleColorsDark*(dst: ptr ImGuiStyle = nil)
    {.importcpp: "ImGui::StyleColorsDark(@)", header: hImGui.}

# Windows ######################################################################

proc beginImGuiWindow*(name: cstring, pOpen: ptr bool = nil, flags: ImGuiWindowFlags = {}): bool
    {.importcpp: "ImGui::Begin(@)", header: hImGui.}

proc endImGuiWindow*()
    {.importcpp: "ImGui::End(@)", header: hImGui.}

# OpenGL3 backend ##############################################################

proc imGuiOpenGl3Setup*(glslVersion: cstring = nil): bool
    {.importcpp: "ImGui_ImplOpenGL3_Init(@)", header: hImGuiImplOpenGl3.}

proc imGuiOpenGl3NewFrame*()
    {.importcpp: "ImGui_ImplOpenGL3_NewFrame(@)", header: hImGuiImplOpenGl3.}

# SDL2 backend #################################################################

proc imGuiSdl2NewFrame*()
    {.importcpp: "ImGui_ImplSDL2_NewFrame(@)", header: hImGuiImplSdl2.}

proc imGuiSdl2OpenGlSetup*(window: ptr SdlWindow, sdlGlContext: pointer): bool
    {.importcpp: "ImGui_ImplSDL2_InitForOpenGL(@)", header: hImGuiImplSdl2.}

proc process*(this: ptr SdlEvent)
    {.importcpp: "ImGui_ImplSDL2_ProcessEvent(@)", header: hImGuiImplSdl2.}
