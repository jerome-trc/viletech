## Functions from dsda-doom's C and C++ code, exposed to Nim.

import std/options

import core, stdx

const hLPrintF = "lprintf.h"
const hSSound = "s_sound.h"
const hWWad = "w_wad.h"

type DBool* = distinct cint

converter toBool*(b: DBool):
    bool = b.cint != 0
converter toDBool*(b: bool): DBool =
    if b: 1.DBool
    else: 0.DBool

# lprintf ######################################################################

type OutputLevels* {.size: sizeof(cint).} = enum
    outlvInfo = 1,
    outlvWarn = 2,
    outlvError = 4,
    outlvDebug = 8,

proc lprintf*(lvl: OutputLevels, fmt: cstring)
    {.importc: "lprintf", varargs, cdecl.}

proc iWarn*(error: cstring)
    {.header: hLPrintF, importc: "I_Warn", varargs, cdecl.}

# Sound ########################################################################

proc changeMusicByName*(cx: ptr CCore, name: cstring, looping: DBool): DBool
    {.importc: "S_ChangeMusicByName", header: hSSound.}

# WAD I/O ######################################################################

type LumpNum* = distinct cint

proc `+`*(x, y: LumpNum): LumpNum {.borrow.}
proc `-`*(x, y: LumpNum): LumpNum {.borrow.}
proc `<`*(x, y: LumpNum): bool {.borrow.}
proc `==`*(x, y: LumpNum): bool {.borrow.}
proc `<=`*(x, y: LumpNum): bool {.borrow.}

type LumpFlags* {.pure, size: sizeof(cint).} = enum
    `static` = 0x00000001
    prboom = 0x00000002
bitFlags(LumpFlags, cint)

type
    WadHeader* {.importc: "wadinfo_t", header: hWWad.} = object
        magic*: array[4, char]
        numLumps*, dirOffs*: cint
    WadDirEntry* {.importc: "filelump_t", header: hWWad.} = object
        filePos*, size*: cint
        name*: array[8, char]
    WadSource* {.
        pure,
        size: sizeof(cint),
        importc: "wad_source_t",
        header: hWWad,
    .} = enum
        skip = -1
        iwad = 0
        pre
        autoLoad
        pwad
        lmp
        net
        deh
        err
    LumpNamespace* {.
        pure,
        size: sizeof(cint),
        importc: "li_namespace_e",
        header: hWWad
    .} = enum
        global = 0
        sprites
        flats
        colormaps
        prboom
        demos
        hires
    WadInfo* {.importc: "wadfile_info_t", header: hWWad.} = object
        name*: cstring
        src*: WadSource
        handle*: cint
    LumpInfo* {.importc: "lumpinfo_t", header: hWWad.} = object
        name*: array[9, char]
        size*, index*, next*: cint
        namespace*: LumpNamespace
        position*: cint
        source*: WadSource
        flags*: LumpFlags
    Lump* = tuple
        info: ptr LumpInfo
        bytes: ptr UncheckedArray[byte]

const lumpNotFound*: LumpNum = -1.LumpNum

var
    lumpInfo* {.global, importc: "lumpinfo", header: hWWad.}: ptr LumpInfo
    numLumps* {.global, importc: "numlumps", header: hWWad.}: LumpNum

proc lumpName*(num: LumpNum): cstring
    {.importc: "W_LumpName", header: hWWad.}

proc lumpLength*(num: LumpNum): cint
    {.importc: "W_LumpLength", header: hWWad.}

proc getLumpBytes*(num: LumpNum): ptr UncheckedArray[byte]
    {.importc: "W_SafeLumpByNum", header: hWWad.}

proc getLumpBytesUnchecked*(num: LumpNum): ptr UncheckedArray[byte]
    {.importc: "W_LumpByNum", header: hWWad.}

proc getLumpInfo*(num: LumpNum): var LumpInfo =
    assert(num >= 0.LumpNum and num < numLumps)
    return lumpInfo.offset(num.int)[]


proc getLump*(num: LumpNum): Option[Lump] =
    let bytes = getLumpBytes(num)

    if bytes.isNil:
        return none(Lump)

    return some((
        info: lumpInfo.offset(num.int),
        bytes: bytes,
    ))


{.push warning[PtrToCstringConv]: off.}

proc name*(self {.byref.}: Lump): cstring =
    assert(self.info.name[8] == '\0')
    return self.info.name[0].addr

{.pop.}

proc `.bytes[]`*(self {.byref.}: Lump, i: Natural): byte =
    assert(i >= 0 and i < self.info.size)
    return self.bytes[i]


proc len*(self {.byref.}: Lump): int = self.info.size
