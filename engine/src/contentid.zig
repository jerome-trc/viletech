// Any code herein taken from the "infer" crate is used under the MIT License.
// See `/legal/infer.txt`.

const std = @import("std");

/// A tag for a file or archive entry representing the result of heuristics used
/// to determine what its content is most likely to be.
pub const ContentId = enum {
    unknown,
    plain_text,

    /// Action Code Script text source. See <https://doomwiki.org/wiki/ACS>.
    acs,
    /// Compiled Action Code Script bytecode blob. See <https://doomwiki.org/wiki/ACS>.
    acs_object,
    /// See <https://zdoom.org/wiki/ALTHUDCF>.
    althudcf,
    /// See <https://doomwiki.org/wiki/ANIMATED>.
    animated,
    /// See <https://zdoom.org/wiki/ANIMDEFS>.
    animdefs,
    /// See, for example, <https://doomwiki.org/wiki/ENDOOM>.
    ansi_text,
    /// See <https://doomwiki.org/wiki/Behavior>.
    behavior,
    /// See <https://doomwiki.org/wiki/Blockmap>.
    blockmap,
    /// See <https://doomwiki.org/wiki/COLORMAP>.
    colormap,
    /// See <https://doomwiki.org/wiki/COMPLVL>.
    complvl,
    /// See <https://zdoom.org/wiki/CVARINFO>.
    cvarinfo,
    /// See <https://zdoom.org/wiki/DECALDEF>.
    decaldef,
    /// See <https://mtrop.github.io/DoomTools/decohack.html>.
    decohack,
    /// See <https://zdoom.org/wiki/DECORATE>.
    decorate,
    /// See <https://zdoom.org/wiki/DEFBINDS>.
    defbinds,
    /// See <https://zdoom.org/wiki/DEFCVARS>.
    defcvars,
    /// See <https://doomwiki.org/wiki/DeHackEd>.
    dehacked,
    /// See [`dehacked`].
    dehacked_bex,
    /// See <https://doomwiki.org/wiki/Demo>.
    demo,
    /// See <https://doomwiki.org/wiki/DEMOLOOP>.
    demoloop,
    /// See <https://doomwiki.org/wiki/DMXGUS>.
    dmxgus,
    /// See <https://doomwiki.org/wiki/MUS>.
    dmx_mus,
    /// Eternity Definition File. See <https://eternity.youfailit.net/wiki/EDF>.
    edf,
    /// Eternity Map Information. See <https://eternity.youfailit.net/wiki/EMAPINFO>.
    emapinfo,
    /// See <https://en.wikipedia.org/wiki/FLAC>.
    flac,
    /// A 64-by-64 texture for floors and ceilings. See <https://doomwiki.org/wiki/Flat>.
    flat,
    /// See <https://zdoom.org/wiki/FONTDEFS>.
    fontdefs,
    /// Global FraggleScript. See <https://zdoom.org/wiki/FraggleScript>.
    fsglobal,
    /// See <https://doomwiki.org/wiki/GAMECONF>.
    gameconf,
    /// See <https://zdoom.org/wiki/GAMEINFO>.
    gameinfo,
    /// See <https://doomwiki.org/wiki/GENMIDI>.
    genmidi,
    /// See <https://zdoom.org/wiki/GLDEFS>.
    gldefs,
    /// See <https://en.wikipedia.org/wiki/OpenGL_Shading_Language>.
    glsl,
    /// See <https://en.wikipedia.org/wiki/High-Level_Shader_Language>.
    hlsl,
    /// See <https://zdoom.org/wiki/IWADINFO>.
    iwadinfo,
    /// See <https://en.wikipedia.org/wiki/JavaScript>.
    javascript, // Likely UDBScript...
    /// See <https://en.wikipedia.org/wiki/JPEG>.
    jpeg,
    /// See <https://en.wikipedia.org/wiki/JPEG_XL>.
    jpeg_xl,
    /// See <https://en.wikipedia.org/wiki/JSON>.
    json,
    /// See <https://zdoom.org/wiki/KEYCONF>.
    keyconf,
    /// See <https://doomwiki.org/wiki/Linedef>.
    linedefs,
    /// See <https://zdoom.org/wiki/LOADACS>.
    loadacs,
    /// See <https://zdoom.org/wiki/LOCKDEFS>.
    lockdefs,
    /// See <https://doomwiki.org/wiki/MAPINFO>.
    mapinfo,
    /// See <https://doomwiki.org/wiki/WAD#Map_data_lumps>.
    map_marker,
    /// See <https://en.wikipedia.org/wiki/Markdown>.
    markdown,
    /// See <https://doomwiki.org/wiki/WAD#Lump_order>.
    marker,
    /// See <https://doomwiki.org/wiki/MIDI>.
    midi,
    /// See <https://en.wikipedia.org/wiki/MP3>.
    mp3,
    /// See <https://zdoom.org/wiki/MODELDEF>.
    modeldef,
    /// See <https://doomwiki.org/wiki/MUSINFO>.
    musinfo,
    /// See <https://doomwiki.org/wiki/Node>.
    nodes,
    /// See <https://zdoom.org/wiki/PALVERS>.
    palvers,
    /// See <https://doomwiki.org/wiki/Picture_format>.
    picture,
    /// See <https://doomwiki.org/wiki/PLAYPAL>.
    playpal,
    /// See <https://doomwiki.org/wiki/PNAMES>.
    pnames,
    /// See <https://en.wikipedia.org/wiki/PNG>.
    png,
    /// See <https://doomwiki.org/wiki/Reject>.
    reject,
    /// See <https://doomwiki.org/wiki/SBARDEF>.
    sbardef,
    /// See <https://zdoom.org/wiki/SBARINFO>.
    sbarinfo,
    /// See <https://doomwiki.org/wiki/Sector>.
    sectors,
    /// See <https://doomwiki.org/wiki/Seg>.
    segs,
    /// See <https://doomwiki.org/wiki/Sidedef>.
    sidedefs,
    /// See <https://doomwiki.org/wiki/SKYDEFS>.
    skydefs,
    /// See <https://zdoom.org/wiki/SNDINFO>.
    sndinfo,
    /// See <https://zdoom.org/wiki/SNDSEQ>.
    sndseq,
    /// See <https://doomwiki.org/wiki/Subsector>.
    ssectors,
    /// See <https://en.wikipedia.org/wiki/SVG>.
    svg,
    /// See <https://doomwiki.org/wiki/SWITCHES>.
    switches,
    /// See <https://zdoom.org/wiki/TERRAIN>.
    terrain,
    /// See <https://zdoom.org/wiki/TEXTCOLO>.
    textcolo,
    /// UDMF TEXTMAP lump. See <https://doomwiki.org/wiki/UDMF>.
    textmap,
    /// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2>.
    texturex,
    /// See <https://doomwiki.org/wiki/Thing>.
    things,
    /// See <https://zdoom.org/wiki/TRNSLATE>.
    trnslate,
    /// Universal Map Information. See <https://doomwiki.org/wiki/UMAPINFO>.
    umapinfo,
    /// See <https://doomwiki.org/wiki/Vertex>.
    vertexes,
    /// See <https://zdoom.org/wiki/VOXELDEF>.
    voxeldef,
    /// An informal standard for level pack metadata.
    wadinfo,
    /// See <https://en.wikipedia.org/wiki/WebP>.
    webp,
    /// See <https://www.w3.org/TR/WGSL/>.
    wgsl,
    /// See <https://zdoom.org/wiki/X11R6RGB>.
    x11r6rgb,
    /// See <https://zdoom.org/wiki/XHAIRS>.
    xhairs,
    /// [`mapinfo`], but ZDoom-specific.
    zmapinfo,
    /// See <https://zdoom.org/wiki/ZScript>.
    zscript,

    pub const by_filestem = std.StaticStringMapWithEql(
        ContentId,
        std.ascii.eqlIgnoreCase,
    ).initComptime(.{
        .{ "althudcf", .althudcf },
        .{ "animated", .animated },
        .{ "animdefs", .animdefs },
        .{ "behavior", .behavior },
        .{ "blockmap", .blockmap },
        .{ "colormap", .colormap },
        .{ "cvarinfo", .cvarinfo },
        .{ "decaldef", .decaldef },
        .{ "decorate", .decorate },
        .{ "defbinds", .defbinds },
        .{ "defcvars", .defcvars },
        .{ "dehacked", .dehacked },
        .{ "demoloop", .demoloop },
        .{ "dmxgus", .dmxgus },
        .{ "dmxgusc", .dmxgus },
        .{ "emapinfo", .emapinfo },
        .{ "emenus", .edf },
        .{ "estrings", .edf },
        .{ "endoom", .ansi_text },
        .{ "fontdefs", .fontdefs },
        .{ "fsglobal", .fsglobal },
        .{ "gameconf", .gameconf },
        .{ "gameinfo", .gameinfo },
        .{ "genmidi", .genmidi },
        .{ "gldefs", .gldefs },
        .{ "iwadinfo", .iwadinfo },
        .{ "keyconf", .keyconf },
        .{ "linedefs", .linedefs },
        .{ "loadacs", .loadacs },
        .{ "lockdefs", .lockdefs },
        .{ "mapinfo", .mapinfo },
        .{ "modeldef", .modeldef },
        .{ "musinfo", .musinfo },
        .{ "nodes", .nodes },
        .{ "palvers", .palvers },
        .{ "playpal", .playpal },
        .{ "pnames", .pnames },
        .{ "reject", .reject },
        .{ "sbarinfo", .sbarinfo },
        .{ "sectors", .sectors },
        .{ "segs", .segs },
        .{ "sidedefs", .sidedefs },
        .{ "sndinfo", .sndinfo },
        .{ "sndseq", .sndseq },
        .{ "ssectors", .ssectors },
        .{ "switches", .switches },
        .{ "terrain", .terrain },
        .{ "textcolo", .textcolo },
        .{ "textmap", .textmap },
        .{ "texture1", .texturex },
        .{ "texture2", .texturex },
        .{ "things", .things },
        .{ "trnslate", .trnslate },
        .{ "umapinfo", .umapinfo },
        .{ "vertexes", .vertexes },
        .{ "voxeldef", .voxeldef },
        .{ "wadinfo", .wadinfo },
        .{ "x11r6rgb", .x11r6rgb },
        .{ "xhairs", .xhairs },
        .{ "zmapinfo", .zmapinfo },
        .{ "zscript", .zscript },
    });

    pub const by_extension = std.StaticStringMapWithEql(
        ContentId,
        std.ascii.eqlIgnoreCase,
    ).initComptime(.{
        .{ ".acs", .acs },
        .{ ".bex", .dehacked_bex },
        .{ ".dec", .decorate },
        .{ ".deh", .dehacked },
        .{ ".dh", .decohack },
        .{ ".edf", .edf },
        .{ ".flac", .flac },
        .{ ".frag", .glsl },
        .{ ".glsl", .glsl },
        .{ ".hlsl", .hlsl },
        .{ ".jfi", .jpeg },
        .{ ".jfif", .jpeg },
        .{ ".jif", .jpeg },
        .{ ".jpe", .jpeg },
        .{ ".jpeg", .jpeg },
        .{ ".jpg", .jpeg },
        .{ ".js", .javascript },
        .{ ".json", .json },
        .{ ".jxl", .jpeg_xl },
        .{ ".md", .markdown },
        .{ ".mid", .midi },
        .{ ".mp3", .mp3 },
        .{ ".mus", .dmx_mus },
        .{ ".svg", .svg },
        .{ ".txt", .plain_text },
        .{ ".vert", .glsl },
        .{ ".webp", .webp },
        .{ ".wgsl", .wgsl },
        .{ ".zs", .zscript },
        .{ ".zsc", .zscript },
    });

    pub fn prettyName(self: ContentId) []const u8 {
        return switch (self) {
            .unknown => "Unknown",
            .plain_text => "Plain text",

            .acs => "ACS source",
            .acs_object => "ACS bytecode",
            .althudcf => "ZDoom alt. HUD config.",
            .animated => "Boom animations",
            .animdefs => "ZDoom animations",
            .ansi_text => "ANSI text",
            .behavior => "ACS map scripts",
            .blockmap => "Blockmap data",
            .colormap => "Colormap",
            .complvl => "Compatibility hint",
            .cvarinfo => "CVar defs",
            .decaldef => "ZDoom decal config.",
            .decorate => "DECORATE",
            .decohack => "DECOHack",
            .defbinds => "ZDoom keybind defaults",
            .defcvars => "ZDoom CVar defaults",
            .dehacked => "DeHackEd",
            .dehacked_bex => "DeH. Boom EXtended",
            .demo => "Demo",
            .demoloop => "ID24 Demo-loop config.",
            .dmx_mus => "Audio (DMX MUS)",
            .dmxgus => "DMX GUS config.",
            .edf => "Eternity definitions",
            .emapinfo => "Eternity map info.",
            .flac => "Audio (FLAC)",
            .flat => "Graphic (Flat)",
            .fontdefs => "ZDoom font config.",
            .fsglobal => "Global FraggleScript",
            .gameconf => "ID24 game config.",
            .gameinfo => "ZDoom game config.",
            .genmidi => "General MIDI set",
            .gldefs => "OpenGL config.",
            .glsl => "OpenGL shader",
            .hlsl => "Direct3D shader",
            .iwadinfo => "ZDoom IWAD config.",
            .javascript => "JavaScript",
            .jpeg => "Graphic (JPEG)",
            .jpeg_xl => "Graphic (JPEG XL)",
            .json => "JSON",
            .keyconf => "ZDoom keybindings",
            .linedefs => "Map lines",
            .loadacs => "ACS loader",
            .lockdefs => "ZDoom lock/key config.",
            .mapinfo => "Map info",
            .map_marker => "Map marker",
            .markdown => "Markdown",
            .marker => "Marker",
            .midi => "Audio (MIDI)",
            .modeldef => "ZDoom model config.",
            .mp3 => "Audio (MP3)",
            .musinfo => "ZDoom music config.",
            .nodes => "Map BSP nodes",
            .palvers => "ZDoom palette versioning",
            .picture => "Graphic (picture)",
            .playpal => "Color palette",
            .pnames => "Patch table",
            .png => "Graphic (PNG)",
            .reject => "Map reject table",
            .sbardef => "ID24 status bar config.",
            .sbarinfo => "ZDoom HUD config.",
            .sectors => "Map sectors",
            .segs => "Map segments",
            .sidedefs => "Map sides",
            .skydefs => "ID24 sky config.",
            .sndinfo => "ZDoom sound config.",
            .sndseq => "ZDoom sound sequence",
            .ssectors => "Map subsectors",
            .svg => "Graphic (SVG)",
            .switches => "Boom switches",
            .terrain => "ZDoom terrain config.",
            .textcolo => "Font colors",
            .textmap => "UDMF textmap",
            .texturex => "TEXTURE1/TEXTURE2",
            .things => "Map object placements",
            .trnslate => "ZDoom translations",
            .umapinfo => "Universal map info",
            .vertexes => "Map vertices",
            .voxeldef => "ZDoom voxel config.",
            .wadinfo => "WAD metadata",
            .webp => "Graphic (WebP)",
            .wgsl => "WebGPU shader",
            .x11r6rgb => "ZDoom X11R6RGB",
            .xhairs => "ZDoom crosshair config.",
            .zmapinfo => "ZDoom map info",
            .zscript => "ZScript",
        };
    }

    pub fn isFlac(buf: []const u8) bool {
        // https://docs.rs/infer/0.16.0/src/infer/matchers/audio.rs.html#49-52
        return buf.len > 3 and std.mem.eql(u8, buf[0..4], "\x66\x4c\x61\x43");
    }

    pub fn isMp3(buf: []const u8) bool {
        // https://docs.rs/infer/0.16.0/src/infer/matchers/audio.rs.html#7-12
        return buf.len > 2 and ((buf[0] == 0x49 and buf[1] == 0x44 and buf[2] == 0x33) // ID3v2
        // Final bit (has crc32) may be or may not be set.
        || (buf[0] == 0xFF and buf[1] == 0xFB));
    }
};

test "ContentId, semantic check" {
    _ = ContentId.by_filestem.get("");
    _ = ContentId.by_extension.get("");
    _ = ContentId.prettyName(.unknown);
}
