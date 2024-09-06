//! Checksums of well-known resources such as DOOM.WAD for use by applications
//! to check whether a given file matches one.

const std = @import("std");

pub const Checksums = struct {
    name: []const u8,
    crc32: u32,
    md5: [16]u8,
    sha1: [20]u8,
};

// DOOM1.WAD ///////////////////////////////////////////////////////////////////

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_0 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0xeedae672,
    .md5 = @bitCast(@as(u128, 0x90facab21eede7981be10790e3f82da2)),
    .sha1 = @bitCast(@as(u160, 0xfc0359e191bd257b3507863ae412ef3250515866)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_1993_12_15 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x289f4d3f,
    .md5 = @bitCast(@as(u128, 0xcea4989df52b65f4d481b706234a3dca)),
    .sha1 = @bitCast(@as(u160, 0x9a24a7093ea0e78fd85f9923e55c55e79491b6a1)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_1 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x981dcebb,
    .md5 = @bitCast(@as(u128, 0x52cbc8882f445573ce421fa5453513c1)),
    .sha1 = @bitCast(@as(u160, 0xd4dc6806abd96bd93570c8df436fb6956e13d910)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_2 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0xbc842626,
    .md5 = @bitCast(@as(u128, 0x30aa5beb9e5ebfbbe1e1765561c08f38)),
    .sha1 = @bitCast(@as(u160, 0x77ef34de7f13dc36b792fb82ed6805e9c1dc7afc)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_25 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x225d7fb1,
    .md5 = @bitCast(@as(u128, 0x17aebd6b5f2ed8ce07aa526a32af8d99)),
    .sha1 = @bitCast(@as(u160, 0x72caf585f7ce56861d25f8580c1cc82bf50abd1b)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_4 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0xf5c2708d,
    .md5 = @bitCast(@as(u128, 0xa21ae40c388cb6f2c3cc1b95589ee693)),
    .sha1 = @bitCast(@as(u160, 0xb4a8e93f1f9544210a173035a0b04c19eb283a2a)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_5 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x8653b0eb,
    .md5 = @bitCast(@as(u128, 0xe280233d533dcc28c1acd6ccdc7742d4)),
    .sha1 = @bitCast(@as(u160, 0xb559ba93d0a96e242eb6ded9deeedbd6f79d40fc)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_6 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0xf26dcad8,
    .md5 = @bitCast(@as(u128, 0x762fd6d4b960d4b759730f01387a50a1)),
    .sha1 = @bitCast(@as(u160, 0x1437fc1ac25a17d5b3cef4c9d2f74e40cae3d231)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_666 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x505fb740,
    .md5 = @bitCast(@as(u128, 0xc428ea394dc52835f2580d5bfd50d76f)),
    .sha1 = @bitCast(@as(u160, 0x81535778d0d4c0c7aa8616fbfd3607dfb3dfd643)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_8 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x331ebf07,
    .md5 = @bitCast(@as(u128, 0x5f4eb849b1af12887dec04a2a12e5e62)),
    .sha1 = @bitCast(@as(u160, 0xc6612ac5a8ac2e2a1d707f9b2869af820efb7c50)),
};

/// https://doomwiki.org/wiki/DOOM1.WAD
pub const doom1_v1_9 = Checksums{
    .name = "DOOM1.WAD",
    .crc32 = 0x162b696a,
    .md5 = @bitCast(@as(u128, 0xf0cefca49926d00903cf57551d901abe)),
    .sha1 = @bitCast(@as(u160, 0x5b2e249b9c5133ec987b3ea77596381dc0d6bc1d)),
};

// TNT.WAD /////////////////////////////////////////////////////////////////////

/// https://doomwiki.org/wiki/TNT.WAD
pub const tnt_v1_9 = Checksums{
    .name = "TNT.WAD",
    .crc32 = 0x903dcc27,
    .md5 = @bitCast(@as(u128, 0x4e158d9953c79ccf97bd0663244cc6b6)),
    .sha1 = @bitCast(@as(u160, 0x9fbc66aedef7fe3bae0986cdb9323d2b8db4c9d3)),
};

/// https://doomwiki.org/wiki/TNT.WAD
pub const tnt_anthology = Checksums{
    .name = "TNT.WAD",
    .crc32 = 0xd4bb05c0,
    .md5 = @bitCast(@as(u128, 0x1d39e405bf6ee3df69a8d2646c8d5c49)),
    .sha1 = @bitCast(@as(u160, 0x4a65c8b960225505187c36040b41a40b152f8f3e)),
};

/// https://doomwiki.org/wiki/TNT.WAD
pub const tnt_psn = Checksums{
    .name = "TNT.WAD",
    .crc32 = 0x7f572c1f,
    .md5 = @bitCast(@as(u128, 0xbe626c12b7c9d94b1dfb9c327566b4ff)),
    .sha1 = @bitCast(@as(u160, 0x139e26d801a64b404b8d898defca10227a61867b)),
};

/// https://doomwiki.org/wiki/TNT.WAD
pub const tnt_kex = Checksums{
    .name = "TNT.WAD",
    .crc32 = 0x15f18ddb,
    .md5 = @bitCast(@as(u128, 0x8974e3117ed4a1839c752d5e11ab1b7b)),
    .sha1 = @bitCast(@as(u160, 0x9820e2a3035f0cdd87f69a7d57c59a7a267c9409)),
};

pub const all = [_]Checksums{
    doom1_v1_0,
    doom1_1993_12_15,
    doom1_v1_1,
    doom1_v1_2,
    doom1_v1_25,
    doom1_v1_4,
    doom1_v1_5,
    doom1_v1_6,
    doom1_v1_666,
    doom1_v1_8,
    doom1_v1_9,

    tnt_v1_9,
    tnt_anthology,
    tnt_psn,
    tnt_kex,
};
