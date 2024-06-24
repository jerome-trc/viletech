import stdx

const hImGuiImplSdl2 = "<imgui_impl_sdl2.h>"
const hSdlVideo = "<SDL_video.h>"

type SdlEvent* {.
    importc: "SDL_Event",
    header: hImGuiImplSdl2,
    union,
    incompleteStruct,
.} = object

type SdlWindow* {.
    importc: "SDL_Window",
    header: hSdlVideo,
    incompleteStruct,
.} = object

type SdlWindowFlags* {.
    pure,
    size: sizeof(cuint),
    importc: "SDL_WindowFlags",
    header: hSdlVideo,
.} = enum
    fullscreen = 0x00000001
    minimized = 0x00000040
bitFlags(SdlWindowFlags, cint)

proc sdlGetWindowFlags*(window: ptr SdlWindow): uint32
    {.importc: "SDL_GetWindowFlags", header: hSdlVideo.}

proc getFlags*(window: ptr SdlWindow): SdlWindowFlags =
    return cast[SdlWindowFlags](sdlGetWindowFlags(window))
