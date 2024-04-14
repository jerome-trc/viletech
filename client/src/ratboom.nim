const libPath = when defined(release):
    "../build/src/Release/libratboom.a"
else:
    "../build/src/Debug/libratboom.a"

{.link: libPath.}
{.passc: "-I./src".}

const projectDir* {.strdefine.} = "."
    ## i.e. `viletech/client`.
