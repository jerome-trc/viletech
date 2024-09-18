const builtin = @import("builtin");
const std = @import("std");

const root = @import("../build.zig");

pub fn build(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    check: *std.Build.Step,
) *std.Build.Step.Compile {
    const cfg_hdr = configHeader(b);
    const datawad = @import("build.data.zig").data(b, target, cfg_hdr);
    var metainfo = b.addOptions();

    const DateTime = root.datetime.DateTime;
    var compile_timestamp_buf: [64]u8 = undefined;
    const compile_timestamp = std.fmt.bufPrint(
        compile_timestamp_buf[0..],
        "{}",
        .{DateTime.now()},
    ) catch unreachable;
    metainfo.addOption([]const u8, "compile_timestamp", compile_timestamp);

    const commit_hash = b.run(&[_][]const u8{ "git", "rev-parse", "HEAD" });
    metainfo.addOption([]const u8, "commit", std.mem.trim(u8, commit_hash, " \n\r\t"));

    const exe = b.addExecutable(.{
        .name = "ratboom",
        .target = target,
        .optimize = optimize,
        .root_source_file = b.path("ratboom/src/main.zig"),
    });
    const exe_check = b.addExecutable(.{
        .name = "ratboom",
        .target = target,
        .optimize = optimize,
        .root_source_file = b.path("ratboom/src/main.zig"),
    });

    const deque = b.createModule(.{ .root_source_file = b.path("depend/deque.zig") });
    exe.root_module.addImport("deque", deque);
    exe_check.root_module.addImport("deque", deque);

    setupExe(b, exe, cfg_hdr, metainfo);
    setupExe(b, exe_check, cfg_hdr, metainfo);
    b.installArtifact(exe);
    check.dependOn(&exe_check.step);

    root.ccdb.createStep(b, "ccdb", .{
        .targets = &[1]*std.Build.Step.Compile{exe},
    });

    exe.step.dependOn(&datawad.step);
    return exe;
}

fn setupExe(
    b: *std.Build,
    exe: *std.Build.Step.Compile,
    cfg_hdr: *std.Build.Step.ConfigHeader,
    metainfo: *std.Build.Step.Options,
) void {
    exe.root_module.addConfigHeader(cfg_hdr);
    exe.step.dependOn(&cfg_hdr.step);
    exe.root_module.addOptions("meta", metainfo);

    exe.linkLibC();
    exe.linkLibCpp();

    exe.addIncludePath(b.path("dsda-doom/prboom2/src"));
    exe.addIncludePath(cfg_hdr.getOutput().dirname());

    const common_flags = [_][]const u8{
        "-ffast-math",
        "-I",
        "build",
        "-I",
        "dsda-doom/prboom2/src",
        "-DHAVE_CONFIG_H",
        "-Dstricmp=strcasecmp",
        "-Dstrnicmp=strncasecmp",
    };

    var c_flags: []const []const u8 = ([_][]const u8{"--std=c99"} ++ common_flags)[0..];
    var cxx_flags: []const []const u8 = ([_][]const u8{"--std=c++20"} ++ common_flags)[0..];

    {
        const flags = b.run(&[_][]const u8{ "pkg-config", "--cflags-only-I", "sdl2" });
        var iter = std.mem.splitScalar(u8, flags, ' ');

        while (iter.next()) |flag| {
            const f = std.mem.trim(u8, flag, " \n\r\t");

            c_flags = std.mem.concat(
                b.allocator,
                []const u8,
                &[2][]const []const u8{ c_flags, &[1][]const u8{f} },
            ) catch unreachable;

            cxx_flags = std.mem.concat(
                b.allocator,
                []const u8,
                &[2][]const []const u8{ cxx_flags, &[1][]const u8{f} },
            ) catch unreachable;
        }
    }

    const c_flags_no_ubsan = std.mem.concat(
        b.allocator,
        []const u8,
        &[2][]const []const u8{ c_flags, &[1][]const u8{"-fno-sanitize=undefined"} },
    ) catch unreachable;

    const cxx_flags_no_ubsan = std.mem.concat(
        b.allocator,
        []const u8,
        &[2][]const []const u8{ cxx_flags, &[1][]const u8{"-fno-sanitize=undefined"} },
    ) catch unreachable;

    exe.addCSourceFiles(.{
        .root = b.path("dsda-doom/prboom2/src"),
        .files = &[_][]const u8{
            "dsda/aim.c",
            "dsda/exdemo.c",
            "dsda/scroll.c",
            "MUSIC/dumbplayer.c",
            "MUSIC/flplayer.c",
            "MUSIC/madplayer.c",
            "MUSIC/midifile.c",
            "MUSIC/opl.c",
            "MUSIC/opl3.c",
            "MUSIC/oplplayer.c",
            "MUSIC/opl_queue.c",
            "MUSIC/portmidiplayer.c",
            "MUSIC/vorbisplayer.c",
            "SDL/i_main.c",
            "SDL/i_sound.c",
            "SDL/i_sshot.c",
            "SDL/i_system.c",
            "SDL/i_video.c",
            "dstrings.c",
            "d_client.c",
            "d_deh.c",
            "d_items.c",
            "d_main.c",
            "e6y.c",
            "f_finale.c",
            "f_wipe.c",
            "gl_clipper.c",
            "gl_drawinfo.c",
            "gl_fbo.c",
            "gl_light.c",
            "gl_main.c",
            "gl_map.c",
            "gl_missingtexture.c",
            "gl_opengl.c",
            "gl_preprocess.c",
            "gl_progress.c",
            "gl_shader.c",
            "gl_sky.c",
            "gl_texture.c",
            "gl_vertex.c",
            "gl_wipe.c",
            "g_game.c",
            "g_overflow.c",
            "heretic/d_main.c",
            "heretic/f_finale.c",
            "heretic/info.c",
            "heretic/in_lude.c",
            "heretic/level_names.c",
            "heretic/mn_menu.c",
            "heretic/sb_bar.c",
            "heretic/sounds.c",
            "hexen/a_action.c",
            "hexen/info.c",
            "hexen/f_finale.c",
            "hexen/h2_main.c",
            "hexen/in_lude.c",
            "hexen/p_acs.c",
            "hexen/p_anim.c",
            "hexen/p_things.c",
            "hexen/po_man.c",
            "hexen/sn_sonix.c",
            "hexen/sounds.c",
            "hexen/sv_save.c",
            "hu_lib.c",
            "hu_stuff.c",
            "info.c",
            "i_capture.c",
            "i_glob.c",
            "lprintf.c",
            "md5.c",
            "memio.c",
            "mus2mid.c",
            "m_argv.c",
            "m_bbox.c",
            "m_cheat.c",
            "m_file.c",
            "m_menu.c",
            "m_misc.c",
            "m_random.c",
            "p_ceilng.c",
            "p_doors.c",
            "p_enemy.c",
            "p_floor.c",
            "p_genlin.c",
            "p_inter.c",
            "p_lights.c",
            "p_map.c",
            "p_maputl.c",
            "p_mobj.c",
            "p_plats.c",
            "p_pspr.c",
            "p_saveg.c",
            "p_setup.c",
            "p_sight.c",
            "p_spec.c",
            "p_switch.c",
            "p_telept.c",
            "p_tick.c",
            "p_user.c",
            "r_bsp.c",
            "r_data.c",
            "r_draw.c",
            "r_fps.c",
            "r_main.c",
            "r_patch.c",
            "r_plane.c",
            "r_segs.c",
            "r_sky.c",
            "r_things.c",
            "sc_man.c",
            "smooth.c",
            "sounds.c",
            "st_lib.c",
            "st_stuff.c",
            "s_advsound.c",
            "s_sound.c",
            "tables.c",
            "v_video.c",
            "wadtbl.c",
            "wi_stuff.c",
            "w_memcache.c",
            "w_wad.c",
            "z_bmalloc.c",
            "z_zone.c",
        },
        .flags = c_flags_no_ubsan,
    });

    exe.addCSourceFiles(.{
        .root = b.path("dsda-doom/prboom2/src"),
        .files = &[_][]const u8{
            "am_map.c",
            "doomdef.c",
            "doomstat.c",
            "dsda.c",
            "dsda/analysis.c",
            "dsda/args.c",
            "dsda/brute_force.c",
            "dsda/build.c",
            "dsda/compatibility.c",
            "dsda/configuration.c",
            "dsda/console.c",
            "dsda/cr_table.c",
            "dsda/data_organizer.c",
            "dsda/death.c",
            "dsda/deh_hash.c",
            "dsda/demo.c",
            "dsda/destructible.c",
            "dsda/endoom.c",
            "dsda/episode.c",
            "dsda/excmd.c",
            "dsda/exhud.c",
            "dsda/features.c",
            "dsda/font.c",
            "dsda/game_controller.c",
            "dsda/ghost.c",
            "dsda/gl/render_scale.c",
            "dsda/global.c",
            "dsda/hud_components/ammo_text.c",
            "dsda/hud_components/armor_text.c",
            "dsda/hud_components/attempts.c",
            "dsda/hud_components/base.c",
            "dsda/hud_components/big_ammo.c",
            "dsda/hud_components/big_armor.c",
            "dsda/hud_components/big_armor_text.c",
            "dsda/hud_components/big_artifact.c",
            "dsda/hud_components/big_health.c",
            "dsda/hud_components/big_health_text.c",
            "dsda/hud_components/color_test.c",
            "dsda/hud_components/command_display.c",
            "dsda/hud_components/composite_time.c",
            "dsda/hud_components/coordinate_display.c",
            "dsda/hud_components/event_split.c",
            "dsda/hud_components/fps.c",
            "dsda/hud_components/free_text.c",
            "dsda/hud_components/health_text.c",
            "dsda/hud_components/keys.c",
            "dsda/hud_components/level_splits.c",
            "dsda/hud_components/line_display.c",
            "dsda/hud_components/line_distance_tracker.c",
            "dsda/hud_components/line_tracker.c",
            "dsda/hud_components/local_time.c",
            "dsda/hud_components/map_coordinates.c",
            "dsda/hud_components/map_time.c",
            "dsda/hud_components/map_title.c",
            "dsda/hud_components/map_totals.c",
            "dsda/hud_components/message.c",
            "dsda/hud_components/minimap.c",
            "dsda/hud_components/mobj_tracker.c",
            "dsda/hud_components/null.c",
            "dsda/hud_components/player_tracker.c",
            "dsda/hud_components/ready_ammo_text.c",
            "dsda/hud_components/render_stats.c",
            "dsda/hud_components/secret_message.c",
            "dsda/hud_components/sector_tracker.c",
            "dsda/hud_components/speed_text.c",
            "dsda/hud_components/stat_totals.c",
            "dsda/hud_components/tracker.c",
            "dsda/hud_components/weapon_text.c",
            "dsda/id_list.c",
            "dsda/input.c",
            "dsda/key_frame.c",
            "dsda/map_format.c",
            "dsda/mapinfo.c",
            "dsda/mapinfo/doom.c",
            "dsda/mapinfo/hexen.c",
            "dsda/mapinfo/legacy.c",
            "dsda/mapinfo/u.c",
            "dsda/memory.c",
            "dsda/messenger.c",
            "dsda/mobjinfo.c",
            "dsda/mouse.c",
            "dsda/msecnode.c",
            "dsda/music.c",
            "dsda/name.c",
            "dsda/options.c",
            "dsda/palette.c",
            "dsda/pause.c",
            "dsda/pclass.c",
            "dsda/playback.c",
            "dsda/preferences.c",
            "dsda/quake.c",
            "dsda/render_stats.c",
            "dsda/save.c",
            "dsda/settings.c",
            "dsda/sfx.c",
            "dsda/skill_info.c",
            "dsda/skip.c",
            "dsda/sndinfo.c",
            "dsda/spawn_number.c",
            "dsda/split_tracker.c",
            "dsda/sprite.c",
            "dsda/state.c",
            "dsda/stretch.c",
            "dsda/text_color.c",
            "dsda/text_file.c",
            "dsda/thing_id.c",
            "dsda/time.c",
            "dsda/tracker.c",
            "dsda/tranmap.c",
            "dsda/utility.c",
            "dsda/utility/string_view.c",
            "dsda/wad_stats.c",
            "dsda/zipfile.c",
        },
        // TODO: the dsda-doom code is riddled with load-bearing UB.
        // Slowly clean it up and add files to this list.
        .flags = c_flags,
    });

    exe.addCSourceFiles(.{
        .root = b.path("dsda-doom/prboom2/src"),
        .files = &[_][]const u8{
            "dsda/ambient.cpp",
            "dsda/mapinfo/doom/parser.cpp",
            "dsda/udmf.cpp",
            "umapinfo.cpp",
        },
        .flags = cxx_flags,
    });

    exe.addCSourceFiles(.{
        .root = b.path("dsda-doom/prboom2/src"),
        .files = &[_][]const u8{
            "scanner.cpp",
        },
        .flags = cxx_flags_no_ubsan,
    });

    const alsa_or_not = if (builtin.os.tag == .linux)
        [_][]const u8{"alsa"}
    else
        [_][]const u8{};

    for ([_][]const u8{
        // "dumb",
        "fluidsynth",
        "GL",
        "GLU",
        "mad",
        "ogg",
        // "portmidi",
        "SDL2-2.0",
        "SDL2_image",
        "SDL2_mixer",
        "sndfile",
        "vorbis",
        "vorbisfile",
        "z",
        "zip",
    } ++ alsa_or_not) |libname| {
        exe.linkSystemLibrary2(libname, .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    root.cimgui.link(b, exe);

    const zig_args = b.dependency("zig-args", .{});
    exe.root_module.addImport("zig-args", zig_args.module("args"));

    root.engine.link(b, exe, null);
}

fn configHeader(b: *std.Build) *std.Build.Step.ConfigHeader {
    const posix_like = switch (builtin.os.tag) {
        .linux => true,
        .windows => false,
        else => @compileError("not yet supported"),
    };

    const wad_dir = if (posix_like)
        "/usr/local/share/games/doom"
    else
        ".";

    const simplechecks = b.option(
        bool,
        "simplecheck",
        "Enable checks which only impose significant overhead if a posible error is detected",
    ) orelse true;

    const rangecheck = b.option(
        bool,
        "rangecheck",
        "Enable extra bounds checks in dsda-doom code",
    ) orelse false;

    return b.addConfigHeader(.{
        .style = .{ .cmake = b.path("dsda-doom/prboom2/cmake/config.h.cin") },
        .include_path = ".zig-cache/config.h",
    }, .{
        .PACKAGE_NAME = "ratboom",
        .PACKAGE_TARNAME = "ratboom",
        .WAD_DATA = "ratboom.wad",
        .PACKAGE_VERSION = "0.0.0",
        .PACKAGE_STRING = "ratboom 0.0.0",

        .VTEC_WAD_DIR = wad_dir,
        .VTEC_ABSOLUTE_PWAD_PATH = wad_dir,

        .WORDS_BIGENDIAN = builtin.cpu.arch.endian() == .big,

        .HAVE_GETOPT = posix_like,
        .HAVE_MMAP = posix_like,
        .HAVE_CREATE_FILE_MAPPING = false,
        .HAVE_STRSIGNAL = posix_like,
        .HAVE_MKSTEMP = posix_like,

        .HAVE_SYS_WAIT_H = posix_like,
        .HAVE_UNISTD_H = posix_like,
        .HAVE_ASM_BYTEORDER_H = posix_like,
        .HAVE_DIRENT_H = posix_like,

        .HAVE_LIBSDL2_IMAGE = true,
        .HAVE_LIBSDL2_MIXER = false,

        .HAVE_LIBDUMB = false,
        .HAVE_LIBFLUIDSYNTH = true,
        .HAVE_LIBMAD = true,
        .HAVE_LIBPORTMIDI = false,
        .HAVE_LIBVORBISFILE = true,

        .SIMPLECHECKS = simplechecks,
        .RANGECHECK = rangecheck,
    });
}
