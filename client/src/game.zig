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

    pub fn prettyName(self: Compat) [:0]const u8 {
        return switch (self) {
            .doom_v1_2 => "Doom v1.2",
            .doom_v1_666 => "Doom v1.666",
            .doom2_v1_9 => "Doom & Doom 2 v1.9",
            .ult_doom => "Ultimate Doom & Doom95",
            .final_doom => "Final Doom",
            .dos_doom => "DosDoom 0.47",
            .tas_doom => "TASDoom",
            .boom_compat => "Boom's Compatibility Mode",
            .boom_v2_01 => "Boom v2.01",
            .boom_v2_02 => "Boom v2.02",
            .lxdoom1 => "LxDoom v1.3.2+",
            .mbf => "Marine's Best Friend",
            .prboom1 => "PrBoom 2.03beta?",
            .prboom2 => "PrBoom 2.1.0-2.1.1",
            .prboom3 => "PrBoom 2.2.x",
            .prboom4 => "PrBoom 2.3.x",
            .prboom5 => "PrBoom 2.4.0",
            .prboom6 => "PrBoom Latest",
            .placeholder18, .placeholder19, .placeholder20 => "",
            .mbf21 => "MBF 21",
        };
    }
};

pub const Skill = enum {
    /// i.e. "I'm too young to die."
    l1,
    /// i.e. "Hey, not too rough."
    l2,
    /// i.e. "Hurt me plenty."
    l3,
    /// i.e. "Ultra-Violence."
    l4,
    /// i.e. "Nightmare!"
    l5,
};

pub const Tick = i32;

pub const Rules = struct {
    compat: Compat,
    skill: Skill,
};
