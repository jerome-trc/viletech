//! An adaption of
//! https://github.com/kraflab/dsda-doom/blob/master/prboom2/data/CMakeLists.txt
//! from CMake to the Zig build system.

const std = @import("std");

pub fn data(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    cfg_hdr: *std.Build.Step.ConfigHeader,
) *std.Build.Step.Run {
    const base_path = "c/data/";
    const rdatawad = b.addExecutable(.{
        .name = "rdatawad",
        .root_source_file = null,
        .target = target,
        .optimize = .ReleaseSafe,
    });
    rdatawad.linkLibC();
    rdatawad.addIncludePath(cfg_hdr.getOutput().dirname());

    rdatawad.addCSourceFiles(.{
        .root = b.path(base_path),
        .flags = &[_][]const u8{},
        .files = &[_][]const u8{
            "rd_main.c",
            "rd_util.c",
            "rd_output.c",
            "rd_sound.c",
            "rd_palette.c",
            "rd_graphic.c",
        },
    });

    const ret = b.addRunArtifact(rdatawad);

    const colormaps = [_][]const u8{
        base_path ++ "lumps/watermap.lmp",
    };

    const flats = [_][]const u8{
        base_path ++ "flats/-n0_tex-.ppm",
    };

    const graphics = [_][]const u8{
        base_path ++ "graphics/dig033.ppm",
        base_path ++ "graphics/dig034.ppm",
        base_path ++ "graphics/dig035.ppm",
        base_path ++ "graphics/dig036.ppm",
        base_path ++ "graphics/dig037.ppm",
        base_path ++ "graphics/dig038.ppm",
        base_path ++ "graphics/dig039.ppm",
        base_path ++ "graphics/dig040.ppm",
        base_path ++ "graphics/dig041.ppm",
        base_path ++ "graphics/dig042.ppm",
        base_path ++ "graphics/dig043.ppm",
        base_path ++ "graphics/dig044.ppm",
        base_path ++ "graphics/dig045.ppm",
        base_path ++ "graphics/dig046.ppm",
        base_path ++ "graphics/dig047.ppm",
        base_path ++ "graphics/dig048.ppm",
        base_path ++ "graphics/dig049.ppm",
        base_path ++ "graphics/dig050.ppm",
        base_path ++ "graphics/dig051.ppm",
        base_path ++ "graphics/dig052.ppm",
        base_path ++ "graphics/dig053.ppm",
        base_path ++ "graphics/dig054.ppm",
        base_path ++ "graphics/dig055.ppm",
        base_path ++ "graphics/dig056.ppm",
        base_path ++ "graphics/dig057.ppm",
        base_path ++ "graphics/dig058.ppm",
        base_path ++ "graphics/dig059.ppm",
        base_path ++ "graphics/dig060.ppm",
        base_path ++ "graphics/dig061.ppm",
        base_path ++ "graphics/dig062.ppm",
        base_path ++ "graphics/dig063.ppm",
        base_path ++ "graphics/dig064.ppm",
        base_path ++ "graphics/dig065.ppm",
        base_path ++ "graphics/dig066.ppm",
        base_path ++ "graphics/dig067.ppm",
        base_path ++ "graphics/dig068.ppm",
        base_path ++ "graphics/dig069.ppm",
        base_path ++ "graphics/dig070.ppm",
        base_path ++ "graphics/dig071.ppm",
        base_path ++ "graphics/dig072.ppm",
        base_path ++ "graphics/dig073.ppm",
        base_path ++ "graphics/dig074.ppm",
        base_path ++ "graphics/dig075.ppm",
        base_path ++ "graphics/dig076.ppm",
        base_path ++ "graphics/dig077.ppm",
        base_path ++ "graphics/dig078.ppm",
        base_path ++ "graphics/dig079.ppm",
        base_path ++ "graphics/dig080.ppm",
        base_path ++ "graphics/dig081.ppm",
        base_path ++ "graphics/dig082.ppm",
        base_path ++ "graphics/dig083.ppm",
        base_path ++ "graphics/dig084.ppm",
        base_path ++ "graphics/dig085.ppm",
        base_path ++ "graphics/dig086.ppm",
        base_path ++ "graphics/dig087.ppm",
        base_path ++ "graphics/dig088.ppm",
        base_path ++ "graphics/dig089.ppm",
        base_path ++ "graphics/dig090.ppm",
        base_path ++ "graphics/dig091.ppm",
        base_path ++ "graphics/dig092.ppm",
        base_path ++ "graphics/dig093.ppm",
        base_path ++ "graphics/dig094.ppm",
        base_path ++ "graphics/dig095.ppm",
        base_path ++ "graphics/dig096.ppm",
        base_path ++ "graphics/dig097.ppm",
        base_path ++ "graphics/dig098.ppm",
        base_path ++ "graphics/dig099.ppm",
        base_path ++ "graphics/dig100.ppm",
        base_path ++ "graphics/dig101.ppm",
        base_path ++ "graphics/dig102.ppm",
        base_path ++ "graphics/dig103.ppm",
        base_path ++ "graphics/dig104.ppm",
        base_path ++ "graphics/dig105.ppm",
        base_path ++ "graphics/dig106.ppm",
        base_path ++ "graphics/dig107.ppm",
        base_path ++ "graphics/dig108.ppm",
        base_path ++ "graphics/dig109.ppm",
        base_path ++ "graphics/dig110.ppm",
        base_path ++ "graphics/dig111.ppm",
        base_path ++ "graphics/dig112.ppm",
        base_path ++ "graphics/dig113.ppm",
        base_path ++ "graphics/dig114.ppm",
        base_path ++ "graphics/dig115.ppm",
        base_path ++ "graphics/dig116.ppm",
        base_path ++ "graphics/dig117.ppm",
        base_path ++ "graphics/dig118.ppm",
        base_path ++ "graphics/dig119.ppm",
        base_path ++ "graphics/dig120.ppm",
        base_path ++ "graphics/dig121.ppm",
        base_path ++ "graphics/dig122.ppm",
        base_path ++ "graphics/dig123.ppm",
        base_path ++ "graphics/dig124.ppm",
        base_path ++ "graphics/dig125.ppm",
        base_path ++ "graphics/dig126.ppm",
        base_path ++ "graphics/boxul.ppm",
        base_path ++ "graphics/boxuc.ppm",
        base_path ++ "graphics/boxur.ppm",
        base_path ++ "graphics/boxcl.ppm",
        base_path ++ "graphics/boxcc.ppm",
        base_path ++ "graphics/boxcr.ppm",
        base_path ++ "graphics/boxll.ppm",
        base_path ++ "graphics/boxlc.ppm",
        base_path ++ "graphics/boxlr.ppm",
        base_path ++ "graphics/stkeys6.ppm",
        base_path ++ "graphics/stkeys7.ppm",
        base_path ++ "graphics/stkeys8.ppm",
        base_path ++ "graphics/stcfn096.ppm",
        base_path ++ "graphics/m_butt1.ppm",
        base_path ++ "graphics/m_butt2.ppm",
        base_path ++ "graphics/m_colors.ppm",
        base_path ++ "graphics/m_palno.ppm",
        base_path ++ "graphics/m_palsel.ppm",
        base_path ++ "graphics/m_vbox.ppm",
        base_path ++ "graphics/cross1.ppm",
        base_path ++ "graphics/cross2.ppm",
        base_path ++ "graphics/cross3.ppm",
        base_path ++ "graphics/cross4.ppm",
        base_path ++ "graphics/cross5.ppm",
        base_path ++ "graphics/cross6.ppm",
        base_path ++ "graphics/cross7.ppm",
    };

    const lumps = [_][]const u8{
        base_path ++ "lumps/switches.lmp",
        base_path ++ "lumps/animated.lmp",
        base_path ++ "lumps/sndcurve.lmp",
        base_path ++ "lumps/soundfont/sndfont.lmp",
        base_path ++ "lumps/dsdacr.lmp",
        base_path ++ "lumps/dsdahud.lmp",
        base_path ++ "lumps/dsdatc.lmp",
    };

    const sounds = [_][]const u8{
        base_path ++ "sounds/dsdgsit.wav",
        base_path ++ "sounds/dsdgatk.wav",
        base_path ++ "sounds/dsdgact.wav",
        base_path ++ "sounds/dsdgdth.wav",
        base_path ++ "sounds/dsdgpain.wav",
        base_path ++ "sounds/dssecret.wav",
    };

    const sprites = [_][]const u8{
        "sprites/tnt1a0.ppm",

        "sprites/dogsa1.ppm",
        "sprites/dogsa2.ppm",
        "sprites/dogsa3.ppm",
        "sprites/dogsa4.ppm",
        "sprites/dogsa5.ppm",
        "sprites/dogsa6.ppm",
        "sprites/dogsa7.ppm",
        "sprites/dogsa8.ppm",
        "sprites/dogsb1.ppm",
        "sprites/dogsb2.ppm",
        "sprites/dogsb3.ppm",
        "sprites/dogsb4.ppm",
        "sprites/dogsb5.ppm",
        "sprites/dogsb6.ppm",
        "sprites/dogsb7.ppm",
        "sprites/dogsb8.ppm",
        "sprites/dogsc1.ppm",
        "sprites/dogsc2.ppm",
        "sprites/dogsc3.ppm",
        "sprites/dogsc4.ppm",
        "sprites/dogsc5.ppm",
        "sprites/dogsc6.ppm",
        "sprites/dogsc7.ppm",
        "sprites/dogsc8.ppm",
        "sprites/dogsd1.ppm",
        "sprites/dogsd2.ppm",
        "sprites/dogsd3.ppm",
        "sprites/dogsd4.ppm",
        "sprites/dogsd5.ppm",
        "sprites/dogsd6.ppm",
        "sprites/dogsd7.ppm",
        "sprites/dogsd8.ppm",
        "sprites/dogse1.ppm",
        "sprites/dogse2.ppm",
        "sprites/dogse3.ppm",
        "sprites/dogse4.ppm",
        "sprites/dogse5.ppm",
        "sprites/dogse6.ppm",
        "sprites/dogse7.ppm",
        "sprites/dogse8.ppm",
        "sprites/dogsf1.ppm",
        "sprites/dogsf2.ppm",
        "sprites/dogsf3.ppm",
        "sprites/dogsf4.ppm",
        "sprites/dogsf5.ppm",
        "sprites/dogsf6.ppm",
        "sprites/dogsf7.ppm",
        "sprites/dogsf8.ppm",
        "sprites/dogsg1.ppm",
        "sprites/dogsg2.ppm",
        "sprites/dogsg3.ppm",
        "sprites/dogsg4.ppm",
        "sprites/dogsg5.ppm",
        "sprites/dogsg6.ppm",
        "sprites/dogsg7.ppm",
        "sprites/dogsg8.ppm",
        "sprites/dogsh1.ppm",
        "sprites/dogsh2.ppm",
        "sprites/dogsh3.ppm",
        "sprites/dogsh4.ppm",
        "sprites/dogsh5.ppm",
        "sprites/dogsh6.ppm",
        "sprites/dogsh7.ppm",
        "sprites/dogsh8.ppm",
        "sprites/dogsi0.ppm",
        "sprites/dogsj0.ppm",
        "sprites/dogsk0.ppm",
        "sprites/dogsl0.ppm",
        "sprites/dogsm0.ppm",
        "sprites/dogsn0.ppm",

        "sprites/pls1a0.ppm",
        "sprites/pls1b0.ppm",
        "sprites/pls1c0.ppm",
        "sprites/pls1d0.ppm",
        "sprites/pls1e0.ppm",
        "sprites/pls1f0.ppm",
        "sprites/pls1g0.ppm",
        "sprites/pls2a0.ppm",
        "sprites/pls2b0.ppm",
        "sprites/pls2c0.ppm",
        "sprites/pls2d0.ppm",
        "sprites/pls2e0.ppm",
    };

    const sprite_p = [_][]const u8{
        "0,0,sprites/tnt1a0.ppm",

        "33,66,sprites/dogsa1.ppm",
        "33,66,sprites/dogsa2.ppm",
        "33,66,sprites/dogsa3.ppm",
        "33,66,sprites/dogsa4.ppm",
        "33,66,sprites/dogsa5.ppm",
        "33,66,sprites/dogsa6.ppm",
        "33,66,sprites/dogsa7.ppm",
        "33,66,sprites/dogsa8.ppm",
        "33,66,sprites/dogsb1.ppm",
        "33,66,sprites/dogsb2.ppm",
        "33,66,sprites/dogsb3.ppm",
        "33,66,sprites/dogsb4.ppm",
        "33,66,sprites/dogsb5.ppm",
        "33,66,sprites/dogsb6.ppm",
        "33,66,sprites/dogsb7.ppm",
        "33,66,sprites/dogsb8.ppm",
        "33,66,sprites/dogsc1.ppm",
        "33,66,sprites/dogsc2.ppm",
        "33,66,sprites/dogsc3.ppm",
        "33,66,sprites/dogsc4.ppm",
        "33,66,sprites/dogsc5.ppm",
        "33,66,sprites/dogsc6.ppm",
        "33,66,sprites/dogsc7.ppm",
        "33,66,sprites/dogsc8.ppm",
        "33,66,sprites/dogsd1.ppm",
        "33,66,sprites/dogsd2.ppm",
        "33,66,sprites/dogsd3.ppm",
        "33,66,sprites/dogsd4.ppm",
        "33,66,sprites/dogsd5.ppm",
        "33,66,sprites/dogsd6.ppm",
        "33,66,sprites/dogsd7.ppm",
        "33,66,sprites/dogsd8.ppm",
        "33,66,sprites/dogse1.ppm",
        "33,66,sprites/dogse2.ppm",
        "33,66,sprites/dogse3.ppm",
        "33,66,sprites/dogse4.ppm",
        "33,66,sprites/dogse5.ppm",
        "33,66,sprites/dogse6.ppm",
        "33,66,sprites/dogse7.ppm",
        "33,66,sprites/dogse8.ppm",
        "33,66,sprites/dogsf1.ppm",
        "33,66,sprites/dogsf2.ppm",
        "33,66,sprites/dogsf3.ppm",
        "33,66,sprites/dogsf4.ppm",
        "33,66,sprites/dogsf5.ppm",
        "33,66,sprites/dogsf6.ppm",
        "33,66,sprites/dogsf7.ppm",
        "33,66,sprites/dogsf8.ppm",
        "33,66,sprites/dogsg1.ppm",
        "33,66,sprites/dogsg2.ppm",
        "33,66,sprites/dogsg3.ppm",
        "33,66,sprites/dogsg4.ppm",
        "33,66,sprites/dogsg5.ppm",
        "33,66,sprites/dogsg6.ppm",
        "33,66,sprites/dogsg7.ppm",
        "33,66,sprites/dogsg8.ppm",
        "33,66,sprites/dogsh1.ppm",
        "33,66,sprites/dogsh2.ppm",
        "33,66,sprites/dogsh3.ppm",
        "33,66,sprites/dogsh4.ppm",
        "33,66,sprites/dogsh5.ppm",
        "33,66,sprites/dogsh6.ppm",
        "33,66,sprites/dogsh7.ppm",
        "33,66,sprites/dogsh8.ppm",
        "33,67,sprites/dogsi0.ppm",
        "33,67,sprites/dogsj0.ppm",
        "33,67,sprites/dogsk0.ppm",
        "33,67,sprites/dogsl0.ppm",
        "33,68,sprites/dogsm0.ppm",
        "33,69,sprites/dogsn0.ppm",

        "9,11,sprites/pls1a0.ppm",
        "8,11,sprites/pls1b0.ppm",
        "9,11,sprites/pls1c0.ppm",
        "8,11,sprites/pls1d0.ppm",
        "16,27,sprites/pls1e0.ppm",
        "16,27,sprites/pls1f0.ppm",
        "18,27,sprites/pls1g0.ppm",
        "9,11,sprites/pls2a0.ppm",
        "8,13,sprites/pls2b0.ppm",
        "11,18,sprites/pls2c0.ppm",
        "16,27,sprites/pls2d0.ppm",
        "18,27,sprites/pls2e0.ppm",
    };

    const tables = [_][]const u8{
        base_path ++ "lumps/sinetabl.lmp",
        base_path ++ "lumps/tangtabl.lmp",
        base_path ++ "lumps/tantoang.lmp",
        base_path ++ "lumps/gammatbl.lmp",
        base_path ++ "lumps/chexdeh.lmp",
        base_path ++ "lumps/bfgbex.lmp",
        base_path ++ "lumps/nervebex.lmp",
        base_path ++ "lumps/glshadow.lmp",
        base_path ++ "lumps/gls_main.lmp",
        base_path ++ "lumps/gls_fuzz.lmp",
        base_path ++ "lumps/gls_v.lmp",
        base_path ++ "lumps/m_ammo.lmp",
        base_path ++ "lumps/m_armour.lmp",
        base_path ++ "lumps/m_arrow.lmp",
        base_path ++ "lumps/m_health.lmp",
        base_path ++ "lumps/m_key.lmp",
        base_path ++ "lumps/m_normal.lmp",
        base_path ++ "lumps/m_shadow.lmp",
        base_path ++ "lumps/m_power.lmp",
        base_path ++ "lumps/m_weap.lmp",
        base_path ++ "lumps/m_mark.lmp",
    };

    ret.addArgs(&[_][]const u8{ "-I", "c/data" });

    ret.addArg("-palette");
    ret.addFileArg(b.path(base_path ++ "palette.rgb"));

    ret.addArg("-lumps");

    for (lumps) |lump| {
        ret.addFileArg(b.path(lump));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "C_START",
        "-lumps",
    });

    for (colormaps) |colormap| {
        ret.addFileArg(b.path(colormap));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "C_END",
    });

    ret.addArgs(&[_][]const u8{
        "-marker",
        "B_START",
        "-lumps",
    });

    for (tables) |table| {
        ret.addFileArg(b.path(table));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "B_END",
    });

    ret.addArg("-sounds");

    for (sounds) |sound| {
        ret.addFileArg(b.path(sound));
    }

    ret.addArg("-graphics");

    for (graphics) |graphic| {
        ret.addFileArg(b.path(graphic));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "FF_START",
        "-flats",
    });

    for (flats) |flat| {
        ret.addFileArg(b.path(flat));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "FF_END",
    });

    ret.addArgs(&[_][]const u8{
        "-marker",
        "SS_START",
        "-sprites",
    });

    for (sprite_p, sprites) |sp, sprite| {
        ret.addArg(sp);
        ret.addFileInput(b.path(sprite));
    }

    ret.addArgs(&[_][]const u8{
        "-marker",
        "SS_END",
    });

    ret.addArgs(&[_][]const u8{
        "-o",
        "zig-out/viletech.wad",
    });

    return ret;
}
