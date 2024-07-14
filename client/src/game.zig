//! Symbols fundamental to the "Doom the game" but not the engine's plumbing.

pub const Compat = enum {
    doom_v1_2,
    doom_v1_666,
    doom2_v1_9,
    ult_doom,
    final_doom,
    dos_doom,
    tas_doom,
    boom_compat,
    boom_v2_01,
    boom_v2_02,
    lxdoom1,
    mbf,
    prboom1,
    prboom2,
    prboom3,
    prboom4,
    prboom5,
    prboom6,
    placeholder18,
    placeholder19,
    placeholder20,
    mbf21,

    const boom = Compat.boom_v2_01;
    const best = Compat.mbf21;

    pub fn boomCompat(self: Compat) bool {
        return @intFromEnum(self) <= @intFromEnum(Compat.boomCompat);
    }

    pub fn demoCompat(self: Compat) bool {
        return @intFromEnum(self) < @intFromEnum(Compat.boomCompat);
    }
};

pub const Tick = i32;
