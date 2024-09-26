const builtin = @import("builtin");
const std = @import("std");

const root = @import("../build.zig");

const Self = @This();

b: *std.Build,
target: std.Build.ResolvedTarget,
optimize: std.builtin.OptimizeMode,
check: *std.Build.Step,

libdumb: bool,
libfluidsynth: bool,
libsdlimage: bool,
libmad: bool,
libportmidi: bool,
libvorbisfile: bool,

assets: *std.Build.Module,
deque: *std.Build.Module,
sdl: *root.Sdl,
zig_args: *std.Build.Module,

pub fn build(self: *Self) *std.Build.Step.Compile {
    const cfg_hdr = configHeader(self);

    const exe_options = std.Build.ExecutableOptions{
        .name = "viletech",
        .root_source_file = self.b.path("client/src/main.zig"),
        .target = self.target,
        .optimize = self.optimize,
    };

    const exe = self.b.addExecutable(exe_options);
    const exe_check = self.b.addExecutable(exe_options);
    self.exeCommon(exe, cfg_hdr);
    self.exeCommon(exe_check, cfg_hdr);

    var metainfo = self.b.addOptions();

    const DateTime = root.datetime.DateTime;
    var compile_timestamp_buf: [64]u8 = undefined;
    const compile_timestamp = std.fmt.bufPrint(
        compile_timestamp_buf[0..],
        "{}",
        .{DateTime.now()},
    ) catch unreachable;
    metainfo.addOption([]const u8, "compile_timestamp", compile_timestamp);

    const commit_hash = self.b.run(&[_][]const u8{ "git", "rev-parse", "HEAD" });
    metainfo.addOption([]const u8, "commit", std.mem.trim(u8, commit_hash, " \n\r\t"));
    metainfo.addOption([]const u8, "version", root.packageVersion());

    exe.root_module.addOptions("meta", metainfo);
    exe_check.root_module.addOptions("meta", metainfo);

    self.b.installArtifact(exe);

    self.check.dependOn(&exe_check.step);

    // Some tools (e.g. zig translate-c) require access to config.h. Put it in a
    // convenient location so there's no need to dig through the .zig-cache.
    const install_cfgh = self.b.addInstallHeaderFile(cfg_hdr.getOutput(), "./config.h");
    exe.step.dependOn(&install_cfgh.step);

    const basedata = @import("build.data.zig").data(self.b, self.target, cfg_hdr);
    exe.step.dependOn(&basedata.step);

    const run_cmd = self.b.addRunArtifact(exe);
    run_cmd.step.dependOn(self.b.getInstallStep());

    if (self.b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = self.b.step("client", "Install and run the VileTech client");
    run_step.dependOn(&run_cmd.step);

    root.ccdb.createStep(self.b, "ccdb", .{
        .targets = &[1]*std.Build.Step.Compile{exe},
    });

    return exe;
}

fn exeCommon(
    self: *Self,
    exe: *std.Build.Step.Compile,
    cfg_hdr: *std.Build.Step.ConfigHeader,
) void {
    exe.root_module.addConfigHeader(cfg_hdr);
    exe.step.dependOn(&cfg_hdr.step);

    exe.addIncludePath(self.b.path("c/src"));
    exe.addIncludePath(cfg_hdr.getOutput().dirname());

    exe.linkLibC();
    exe.linkLibCpp();

    root.engine.link(self.b, exe, null);
    root.subterra.link(self.b, exe, .{ .znbx = .source });
    root.wadload.link(self.b, exe, null);
    self.sdl.link(exe, .static, .SDL2);
    root.zmsx.link(self.b, exe, exe.root_module.resolved_target.?, exe.root_module.optimize.?);

    exe.root_module.addImport("assets", self.assets);
    exe.root_module.addImport("deque", self.deque);
    exe.root_module.addImport("sdl", self.sdl.getWrapperModule());
    exe.root_module.addImport("zig-args", self.zig_args);

    const common_flags = [_][]const u8{
        "-ffast-math",
        "-I",
        "build",
        "-I",
        "c/src",
        "-DHAVE_CONFIG_H",
        "-Dstricmp=strcasecmp",
        "-Dstrnicmp=strncasecmp",
    };

    var c_flags: []const []const u8 = ([_][]const u8{"--std=c99"} ++ common_flags)[0..];
    var cxx_flags: []const []const u8 = ([_][]const u8{"--std=c++20"} ++ common_flags)[0..];

    {
        const flags = self.b.run(&[_][]const u8{ "pkg-config", "--cflags-only-I", "sdl2" });
        var iter = std.mem.splitScalar(u8, flags, ' ');

        while (iter.next()) |flag| {
            const f = std.mem.trim(u8, flag, " \n\r\t");

            c_flags = std.mem.concat(
                self.b.allocator,
                []const u8,
                &[2][]const []const u8{ c_flags, &[1][]const u8{f} },
            ) catch unreachable;

            cxx_flags = std.mem.concat(
                self.b.allocator,
                []const u8,
                &[2][]const []const u8{ cxx_flags, &[1][]const u8{f} },
            ) catch unreachable;
        }
    }

    const c_flags_no_ubsan = std.mem.concat(
        self.b.allocator,
        []const u8,
        &[2][]const []const u8{ c_flags, &[1][]const u8{"-fno-sanitize=undefined"} },
    ) catch unreachable;

    const cxx_flags_no_ubsan = std.mem.concat(
        self.b.allocator,
        []const u8,
        &[2][]const []const u8{ cxx_flags, &[1][]const u8{"-fno-sanitize=undefined"} },
    ) catch unreachable;

    exe.addCSourceFiles(.{
        .root = self.b.path("c/src"),
        .files = &[_][]const u8{
            "dsda/aim.c",
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
            "dsda/exdemo.c",
            "dsda/exhud.c",
            "dsda/features.c",
            "dsda/font.c",
            "dsda/game_controller.c",
            "dsda/ghost.c",
            "dsda/gl/render_scale.c",
            "dsda/global.c",
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
            "dsda/scroll.c",
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

            "heretic/d_main.c",
            "heretic/f_finale.c",
            "heretic/in_lude.c",
            "heretic/info.c",
            "heretic/level_names.c",
            "heretic/mn_menu.c",
            "heretic/sb_bar.c",
            "heretic/sounds.c",

            "hexen/a_action.c",
            "hexen/f_finale.c",
            "hexen/h2_main.c",
            "hexen/in_lude.c",
            "hexen/info.c",
            "hexen/p_acs.c",
            "hexen/p_anim.c",
            "hexen/p_things.c",
            "hexen/po_man.c",
            "hexen/sn_sonix.c",
            "hexen/sounds.c",
            "hexen/sv_save.c",

            "MUSIC/dumbplayer.c",
            "MUSIC/flplayer.c",
            "MUSIC/madplayer.c",
            "MUSIC/midifile.c",
            "MUSIC/opl_queue.c",
            "MUSIC/opl.c",
            "MUSIC/opl3.c",
            "MUSIC/oplplayer.c",
            "MUSIC/portmidiplayer.c",
            "MUSIC/vorbisplayer.c",

            "SDL/i_main.c",
            "SDL/i_sound.c",
            "SDL/i_sshot.c",
            "SDL/i_system.c",
            "SDL/i_video.c",

            "am_map.c",
            "d_client.c",
            "d_deh.c",
            "d_items.c",
            "d_main.c",
            "doomdef.c",
            "doomstat.c",
            "dsda.c",
            "dstrings.c",
            "e6y.c",
            "f_finale.c",
            "f_wipe.c",
            "g_game.c",
            "g_overflow.c",
            "gl_texture.c",
            "hu_lib.c",
            "hu_stuff.c",
            "i_capture.c",
            "i_glob.c",
            "info.c",
            "lprintf.c",
            "m_argv.c",
            "m_bbox.c",
            "m_cheat.c",
            "m_file.c",
            "m_menu.c",
            "m_misc.c",
            "m_random.c",
            "md5.c",
            "memio.c",
            "mus2mid.c",
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
            "s_advsound.c",
            "s_sound.c",
            "sc_man.c",
            "smooth.c",
            "sounds.c",
            "st_lib.c",
            "st_stuff.c",
            "tables.c",
            "v_video.c",
            "w_memcache.c",
            "w_wad.c",
            "wadtbl.c",
            "wi_stuff.c",
            "z_bmalloc.c",
            "z_zone.c",
        },
        .flags = c_flags_no_ubsan,
    });

    exe.addCSourceFiles(.{
        .root = self.b.path("c/src"),
        .files = &[_][]const u8{
            "dsda/hud_components/ammo_text.c",
            "dsda/hud_components/armor_text.c",
            "dsda/hud_components/attempts.c",
            "dsda/hud_components/base.c",
            "dsda/hud_components/big_ammo.c",
            "dsda/hud_components/big_armor_text.c",
            "dsda/hud_components/big_armor.c",
            "dsda/hud_components/big_artifact.c",
            "dsda/hud_components/big_health_text.c",
            "dsda/hud_components/big_health.c",
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
            "gl_vertex.c",
            "gl_wipe.c",
        },
        // TODO: the dsda-doom code is riddled with load-bearing UB.
        // Slowly clean it up and add files to this list.
        .flags = c_flags,
    });

    exe.addCSourceFile(.{
        .file = self.b.path("c/src/p_tick.c"),
        // Don't build this file as C99 or it miscompiles during Clang optimizations.
        // Only seems to appear during the "e1 sk4 max in 45:37 by PVS" demotest,
        // likely related to prototype-less C function pointers.
        .flags = &common_flags,
    });

    exe.addCSourceFiles(.{
        .root = self.b.path("c/src"),
        .files = &[_][]const u8{
            "dsda/ambient.cpp",
            "dsda/mapinfo/doom/parser.cpp",
            "dsda/udmf.cpp",
            "umapinfo.cpp",
        },
        .flags = cxx_flags,
    });

    exe.addCSourceFiles(.{
        .root = self.b.path("c/src"),
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
        "flac",
        "GL",
        "GLU",
        "ogg",
        "opus",
        "SDL2-2.0",
        "SDL2_mixer",
        "sndfile",
        "z",
        "zip",
    } ++ alsa_or_not) |libname| {
        exe.linkSystemLibrary2(libname, .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    if (self.libdumb) {
        exe.linkSystemLibrary2("dumb", .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    if (self.libfluidsynth) {
        exe.linkSystemLibrary2("fluidsynth", .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    if (self.libmad) {
        exe.linkSystemLibrary2("mad", .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    if (self.libportmidi) {
        exe.linkSystemLibrary2("portmidi", .{
            .needed = true,
            .preferred_link_mode = .dynamic,
            .use_pkg_config = .yes,
        });
    }

    if (self.libsdlimage) {
        exe.linkSystemLibrary2("SDL2_image", .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }

    if (self.libvorbisfile) {
        for ([_][]const u8{ "vorbis", "vorbisenc", "vorbisfile" }) |libname| {
            exe.linkSystemLibrary2(libname, .{
                .needed = true,
                .preferred_link_mode = .static,
                .use_pkg_config = .yes,
            });
        }
    }
}

fn configHeader(self: *Self) *std.Build.Step.ConfigHeader {
    const posix_like = switch (builtin.os.tag) {
        .linux => true,
        .windows => false,
        else => @compileError("not yet supported"),
    };

    const wad_dir = if (posix_like)
        "/usr/local/share/games/doom"
    else
        ".";

    const simplechecks = self.b.option(
        bool,
        "simplecheck",
        "Enable checks which only impose significant overhead if a posible error is detected",
    ) orelse true;

    const rangecheck = self.b.option(
        bool,
        "rangecheck",
        "Enable extra bounds checks in C code",
    ) orelse false;

    return self.b.addConfigHeader(.{
        .style = .{ .cmake = self.b.path("c/cmake/config.h.cin") },
        .include_path = ".zig-cache/config.h",
    }, .{
        .PACKAGE_NAME = "viletech",
        .PACKAGE_TARNAME = "viletech",
        .WAD_DATA = "viletech.wad",
        .PACKAGE_VERSION = root.packageVersion(),
        .PACKAGE_STRING = "viletech " ++ comptime root.packageVersion(),

        .VTEC_WAD_DIR = wad_dir,
        .VTEC_ABSOLUTE_PWAD_PATH = wad_dir,

        .WORDS_BIGENDIAN = builtin.cpu.arch.endian() == .big,

        .HAVE_GETOPT = posix_like,
        .HAVE_MMAP = posix_like,
        .HAVE_CREATE_FILE_MAPPING = builtin.os.tag == .windows,
        .HAVE_STRSIGNAL = posix_like,
        .HAVE_MKSTEMP = posix_like,

        .HAVE_SYS_WAIT_H = posix_like,
        .HAVE_UNISTD_H = posix_like,
        .HAVE_ASM_BYTEORDER_H = posix_like,
        .HAVE_DIRENT_H = posix_like,

        // TODO: detection for these. Is it possible to do better than just pkg-config?
        .HAVE_LIBSDL2_IMAGE = self.libsdlimage,
        .HAVE_LIBSDL2_MIXER = true,

        .HAVE_LIBDUMB = self.libdumb,
        .HAVE_LIBFLUIDSYNTH = self.libfluidsynth,
        .HAVE_LIBMAD = self.libmad,
        .HAVE_LIBPORTMIDI = self.libportmidi,
        .HAVE_LIBVORBISFILE = self.libvorbisfile,

        .SIMPLECHECKS = simplechecks,
        .RANGECHECK = rangecheck,
    });
}
