when defined(nimHasUsed):
    {.used.}

{.passc: "-I../build".}
{.passc: "-I./src".}

const dsdaLibPath = when defined(release):
    "../build/src/Release/libratboom.a"
else:
    "../build/src/Debug/libratboom.a"

{.link: dsdaLibPath.}

const projectDir* {.strdefine.} = "."
    ## i.e. `viletech/client`.
