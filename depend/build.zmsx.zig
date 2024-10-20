const builtin = @import("builtin");
const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const dep = b.dependency("zmsx", .{});

    var lib = b.addStaticLibrary(.{
        .name = "zmsx",
        .target = target,
        .optimize = optimize,
    });
    lib.linkLibC();
    lib.linkLibCpp();

    const fast_math = [_][]const u8{ "-ffast-math", "-ffp-contract=fast" };
    const stricmp = [_][]const u8{"-Dstricmp=strcasecmp"};
    const strnicmp = [_][]const u8{"-Dstrnicmp=strncasecmp"};

    var src_flags: []const []const u8 = &(fast_math ++ stricmp ++ strnicmp ++ [_][]const u8{
        "-I",
        "source",
        "-I",
        "source/zmsx",
        "-I",
        "include",
        "-isystem",
        "thirdparty/adlmidi",
        "-isystem",
        "thirdparty/game-music-emu",
        "-isystem",
        "thirdparty/miniz",
        "-isystem",
        "thirdparty/oplsynth",
        "-isystem",
        "thirdparty/opnmidi",
        "-isystem",
        "thirdparty/timidity",
        "-isystem",
        "thirdparty/timidityplus",
        "-isystem",
        "thirdparty/wildmidi",
        "-DHAVE_SNDFILE",
    });

    {
        const flags = b.run(&[_][]const u8{ "pkg-config", "--cflags", "sndfile" });
        var iter = std.mem.splitScalar(u8, flags, ' ');

        while (iter.next()) |flag| {
            src_flags = std.mem.concat(
                b.allocator,
                []const u8,
                &[2][]const []const u8{ src_flags, &[1][]const u8{flag} },
            ) catch unreachable;
        }
    }

    lib.addCSourceFiles(.{
        .root = dep.path("."),
        .flags = src_flags,
        .files = &[_][]const u8{
            "source/loader/i_module.cpp",
            "source/mididevices/music_base_mididevice.cpp",
            "source/mididevices/music_adlmidi_mididevice.cpp",
            "source/mididevices/music_opl_mididevice.cpp",
            "source/mididevices/music_opnmidi_mididevice.cpp",
            "source/mididevices/music_timiditypp_mididevice.cpp",
            "source/mididevices/music_fluidsynth_mididevice.cpp",
            "source/mididevices/music_softsynth_mididevice.cpp",
            "source/mididevices/music_timidity_mididevice.cpp",
            "source/mididevices/music_wildmidi_mididevice.cpp",
            "source/mididevices/music_wavewriter_mididevice.cpp",
            "source/midisources/midisource.cpp",
            "source/midisources/midisource_mus.cpp",
            "source/midisources/midisource_smf.cpp",
            "source/midisources/midisource_hmi.cpp",
            "source/midisources/midisource_xmi.cpp",
            "source/midisources/midisource_mids.cpp",
            "source/streamsources/music_dumb.cpp",
            "source/streamsources/music_gme.cpp",
            "source/streamsources/music_libsndfile.cpp",
            "source/streamsources/music_opl.cpp",
            "source/streamsources/music_xa.cpp",
            "source/musicformats/music_stream.cpp",
            "source/musicformats/music_midi.cpp",
            "source/musicformats/music_cd.cpp",
            "source/decoder/sounddecoder.cpp",
            "source/decoder/sndfile_decoder.cpp",
            "source/decoder/mpg123_decoder.cpp",
            "source/zmsx/configuration.cpp",
            "source/zmsx/zmsx.cpp",
            "source/zmsx/critsec.cpp",
            "source/loader/test.c",
        },
    });

    lib.addIncludePath(dep.path("include"));
    lib.addIncludePath(dep.path("source"));
    lib.addIncludePath(dep.path("source/zmsx"));
    lib.addSystemIncludePath(dep.path("thirdparty/adlmidi"));
    lib.addSystemIncludePath(dep.path("thirdparty/dumb"));
    lib.addSystemIncludePath(dep.path("thirdparty/game-music-emu"));
    lib.addSystemIncludePath(dep.path("thirdparty/miniz"));
    lib.addSystemIncludePath(dep.path("thirdparty/oplsynth"));
    lib.addSystemIncludePath(dep.path("thirdparty/opnmidi"));
    lib.addSystemIncludePath(dep.path("thirdparty/timidity"));
    lib.addSystemIncludePath(dep.path("thirdparty/timidityplus"));
    lib.addSystemIncludePath(dep.path("thirdparty/wildmidi"));

    {
        const adlmidi = b.addStaticLibrary(.{
            .name = "adlmidi",
            .target = target,
            .optimize = optimize,
        });

        adlmidi.addCSourceFiles(.{
            .root = dep.path("thirdparty/adlmidi"),
            .flags = &(fast_math ++ [_][]const u8{
                "-DADLMIDI_DISABLE_MIDI_SEQUENCER",
            }),
            .files = &[_][]const u8{
                "adlmidi_midiplay.cpp",
                "adlmidi_opl3.cpp",
                "adlmidi_private.cpp",
                "adlmidi.cpp",
                "adlmidi_load.cpp",
                "inst_db.cpp",
                "chips/opal_opl3.cpp",
                "chips/dosbox/dbopl.cpp",
                "chips/nuked_opl3_v174.cpp",
                "chips/java_opl3.cpp",
                "chips/dosbox_opl3.cpp",
                "chips/nuked_opl3.cpp",
                "chips/nuked/nukedopl3_174.c",
                "chips/nuked/nukedopl3.c",
                "wopl/wopl_file.c",
            },
        });

        adlmidi.linkLibC();
        adlmidi.linkLibCpp();
        adlmidi.addIncludePath(dep.path("thirdparty/adlmidi"));

        compile.linkLibrary(adlmidi);
    }

    {
        const dumb = b.addStaticLibrary(.{
            .name = "dumb",
            .target = target,
            .optimize = optimize,
        });

        dumb.addCSourceFiles(.{
            .root = dep.path("thirdparty/dumb"),
            .flags = &(fast_math ++ [_][]const u8{
                // "-msse",
                "-DNEED_ITOA=1",
            }),
            .files = &[_][]const u8{
                "src/core/unload.c",
                "src/core/rendsig.c",
                "src/core/rendduh.c",
                "src/core/register.c",
                "src/core/readduh.c",
                "src/core/rawsig.c",
                "src/core/makeduh.c",
                "src/core/loadduh.c",
                "src/core/dumbfile.c",
                "src/core/duhtag.c",
                "src/core/duhlen.c",
                "src/core/atexit.c",
                "src/helpers/stdfile.c",
                "src/helpers/silence.c",
                "src/helpers/sampbuf.c",
                "src/helpers/riff.c",
                "src/helpers/resample.c",
                "src/helpers/memfile.c",
                "src/helpers/clickrem.c",
                "src/helpers/barray.c",
                "src/it/xmeffect.c",
                "src/it/readxm2.c",
                "src/it/readxm.c",
                "src/it/readstm2.c",
                "src/it/readstm.c",
                "src/it/reads3m2.c",
                "src/it/reads3m.c",
                "src/it/readriff.c",
                "src/it/readptm.c",
                "src/it/readpsm.c",
                "src/it/readoldpsm.c",
                "src/it/readokt2.c",
                "src/it/readokt.c",
                "src/it/readmtm.c",
                "src/it/readmod2.c",
                "src/it/readmod.c",
                "src/it/readdsmf.c",
                "src/it/readasy.c",
                "src/it/readamf2.c",
                "src/it/readamf.c",
                "src/it/readam.c",
                "src/it/read6692.c",
                "src/it/read669.c",
                "src/it/ptmeffect.c",
                "src/it/loadxm2.c",
                "src/it/loadxm.c",
                "src/it/loadstm2.c",
                "src/it/loadstm.c",
                "src/it/loads3m2.c",
                "src/it/loads3m.c",
                "src/it/loadriff2.c",
                "src/it/loadriff.c",
                "src/it/loadptm2.c",
                "src/it/loadptm.c",
                "src/it/loadpsm2.c",
                "src/it/loadpsm.c",
                "src/it/loadoldpsm2.c",
                "src/it/loadoldpsm.c",
                "src/it/loadokt2.c",
                "src/it/loadokt.c",
                "src/it/loadmtm2.c",
                "src/it/loadmtm.c",
                "src/it/loadmod2.c",
                "src/it/loadmod.c",
                "src/it/loadasy2.c",
                "src/it/loadasy.c",
                "src/it/loadamf2.c",
                "src/it/loadamf.c",
                "src/it/load6692.c",
                "src/it/load669.c",
                "src/it/itunload.c",
                "src/it/itrender.c",
                "src/it/itread2.c",
                "src/it/itread.c",
                "src/it/itorder.c",
                "src/it/itmisc.c",
                "src/it/itload2.c",
                "src/it/itload.c",
                "src/it/readany.c",
                "src/it/loadany2.c",
                "src/it/loadany.c",
                "src/it/readany2.c",
                "src/helpers/resampler.c",
                "src/helpers/lpc.c",
            },
        });

        dumb.linkLibC();
        dumb.addIncludePath(dep.path("thirdparty/dumb/include"));

        compile.linkLibrary(dumb);
    }

    {
        var fluidsynth_flags: []const []const u8 = &[_][]const u8{};

        if (builtin.os.tag != .windows) {
            const flags = b.run(&[_][]const u8{ "pkg-config", "--cflags", "glib-2.0" });
            var iter = std.mem.splitScalar(u8, flags, ' ');

            while (iter.next()) |flag| {
                const f = std.mem.trim(u8, flag, " \n\r\t");

                fluidsynth_flags = std.mem.concat(
                    b.allocator,
                    []const u8,
                    &[2][]const []const u8{ fluidsynth_flags, &[1][]const u8{f} },
                ) catch unreachable;
            }
        }

        const fluidsynth = b.addStaticLibrary(.{
            .name = "fluidsynth",
            .target = target,
            .optimize = optimize,
        });

        fluidsynth.linkLibC();
        fluidsynth.linkLibCpp();
        fluidsynth.addIncludePath(dep.path("source/decoder"));
        fluidsynth.addIncludePath(dep.path("thirdparty"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/include"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/bindings"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/drivers"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/midi"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/rvoice"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/sfloader"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/synth"));
        fluidsynth.addIncludePath(dep.path("thirdparty/fluidsynth/src/utils"));

        fluidsynth.addCSourceFiles(.{
            .root = dep.path("."),
            .flags = fluidsynth_flags,
            .files = &[_][]const u8{
                "thirdparty/fluidsynth/src/utils/fluid_conv.c",
                "thirdparty/fluidsynth/src/utils/fluid_hash.c",
                "thirdparty/fluidsynth/src/utils/fluid_list.c",
                "thirdparty/fluidsynth/src/utils/fluid_ringbuffer.c",
                "thirdparty/fluidsynth/src/utils/fluid_settings.c",
                "thirdparty/fluidsynth/src/utils/fluid_sys.c",
                "thirdparty/fluidsynth/src/sfloader/fluid_defsfont.c",
                "thirdparty/fluidsynth/src/sfloader/fluid_sfont.c",
                "thirdparty/fluidsynth/src/sfloader/fluid_sffile.c",
                "thirdparty/fluidsynth/src/sfloader/fluid_samplecache.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_adsr_env.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_chorus.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_iir_filter.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_lfo.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_rvoice.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_rvoice_dsp.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_rvoice_event.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_rvoice_mixer.c",
                "thirdparty/fluidsynth/src/rvoice/fluid_rev.c",
                "thirdparty/fluidsynth/src/synth/fluid_chan.c",
                "thirdparty/fluidsynth/src/synth/fluid_event.c",
                "thirdparty/fluidsynth/src/synth/fluid_gen.c",
                "thirdparty/fluidsynth/src/synth/fluid_mod.c",
                "thirdparty/fluidsynth/src/synth/fluid_synth.c",
                "thirdparty/fluidsynth/src/synth/fluid_synth_monopoly.c",
                "thirdparty/fluidsynth/src/synth/fluid_tuning.c",
                "thirdparty/fluidsynth/src/synth/fluid_voice.c",
                "thirdparty/fluidsynth/src/midi/fluid_midi.c",
                "thirdparty/fluidsynth/src/midi/fluid_midi_router.c",
                "thirdparty/fluidsynth/src/midi/fluid_seqbind.c",
                "thirdparty/fluidsynth/src/midi/fluid_seqbind_notes.cpp",
                "thirdparty/fluidsynth/src/midi/fluid_seq.c",
                "thirdparty/fluidsynth/src/midi/fluid_seq_queue.cpp",
                "thirdparty/fluidsynth/src/drivers/fluid_adriver.c",
                "thirdparty/fluidsynth/src/drivers/fluid_mdriver.c",
                "thirdparty/fluidsynth/src/bindings/fluid_filerenderer.c",
                "thirdparty/fluidsynth/src/bindings/fluid_ladspa.c",
            },
        });

        if (builtin.os.tag != .windows) {
            compile.linkSystemLibrary2("glib-2.0", .{
                .needed = true,
                .preferred_link_mode = .static,
                .use_pkg_config = .yes,
            });
        }

        compile.linkLibrary(fluidsynth);
    }

    {
        const gme = b.addStaticLibrary(.{
            .name = "gme",
            .target = target,
            .optimize = optimize,
        });

        gme.addCSourceFiles(.{
            .root = dep.path("thirdparty"),
            .flags = &(fast_math ++ [_][]const u8{
                "-fomit-frame-pointer",
                "-fwrapv",
                "-DHAVE_ZLIB_H",
            }),
            .files = &[_][]const u8{
                "game-music-emu/gme/Blip_Buffer.cpp",
                "game-music-emu/gme/Classic_Emu.cpp",
                "game-music-emu/gme/Data_Reader.cpp",
                "game-music-emu/gme/Dual_Resampler.cpp",
                "game-music-emu/gme/Effects_Buffer.cpp",
                "game-music-emu/gme/Fir_Resampler.cpp",
                "game-music-emu/gme/gme.cpp",
                "game-music-emu/gme/Gme_File.cpp",
                "game-music-emu/gme/M3u_Playlist.cpp",
                "game-music-emu/gme/Multi_Buffer.cpp",
                "game-music-emu/gme/Music_Emu.cpp",
                // TODO: optional emulators?
            },
        });

        gme.linkLibC();
        gme.linkLibCpp();
        gme.addIncludePath(dep.path("game-music-emu/gme"));
        gme.addSystemIncludePath(dep.path("thirdparty/miniz"));

        compile.linkLibrary(gme);
    }

    {
        const miniz = b.addStaticLibrary(.{
            .name = "miniz",
            .target = target,
            .optimize = optimize,
        });

        miniz.addCSourceFiles(.{
            .root = dep.path("thirdparty/miniz"),
            .flags = &[_][]const u8{},
            .files = &[_][]const u8{"miniz.c"},
        });

        miniz.linkLibC();
        miniz.addIncludePath(dep.path("thirdparty/miniz"));

        compile.linkLibrary(miniz);
    }

    {
        const oplsynth = b.addStaticLibrary(.{
            .name = "oplsynth",
            .target = target,
            .optimize = optimize,
        });

        oplsynth.addCSourceFiles(.{
            .root = dep.path("thirdparty/oplsynth"),
            .flags = &(fast_math ++ stricmp ++ strnicmp ++ [_][]const u8{
                "-I",
                ".",
                "-I",
                "oplsynth",
                "-fomit-frame-pointer",
            }),
            .files = &[_][]const u8{
                "fmopl.cpp",
                "musicblock.cpp",
                "nukedopl3.cpp",
                "opl_mus_player.cpp",
                "OPL3.cpp",
                "oplio.cpp",
                "dosbox/opl.cpp",
            },
        });

        oplsynth.linkLibC();
        oplsynth.linkLibCpp();
        oplsynth.addIncludePath(dep.path("thirdparty/oplsynth"));
        oplsynth.addIncludePath(dep.path("thirdparty/oplsynth/oplsynth"));

        compile.linkLibrary(oplsynth);
    }

    {
        const opnmidi = b.addStaticLibrary(.{
            .name = "opnmidi",
            .target = target,
            .optimize = optimize,
        });

        opnmidi.addCSourceFiles(.{
            .root = dep.path("thirdparty/opnmidi"),
            .flags = &(fast_math ++ [_][]const u8{
                "-DOPNMIDI_DISABLE_MIDI_SEQUENCER",
                "-DOPNMIDI_DISABLE_GX_EMULATOR",
            }),
            .files = &[_][]const u8{
                "opnmidi_load.cpp",
                "opnmidi_private.cpp",
                "opnmidi.cpp",
                "opnmidi_midiplay.cpp",
                "opnmidi_opn2.cpp",
                "chips/np2/fmgen_fmgen.cpp",
                "chips/np2/fmgen_opna.cpp",
                "chips/np2/fmgen_fmtimer.cpp",
                "chips/np2/fmgen_file.cpp",
                "chips/np2/fmgen_psg.cpp",
                "chips/mame_opn2.cpp",
                "chips/gens_opn2.cpp",
                "chips/mame_opna.cpp",
                "chips/np2_opna.cpp",
                "chips/mamefm/ymdeltat.cpp",
                "chips/mamefm/resampler.cpp",
                "chips/mamefm/fm.cpp",
                "chips/nuked_opn2.cpp",
                "chips/gens/Ym2612.cpp",
                "chips/gx_opn2.cpp",
                "chips/pmdwin_opna.cpp",
                "chips/nuked/ym3438.c",
                "chips/gx/gx_ym2612.c",
                "chips/pmdwin/opna.c",
                "chips/pmdwin/psg.c",
                "chips/pmdwin/rhythmdata.c",
                "chips/mamefm/emu2149.c",
                "chips/mame/mame_ym2612fm.c",
                "wopn/wopn_file.c",
            },
        });

        opnmidi.linkLibC();
        opnmidi.linkLibCpp();
        opnmidi.addIncludePath(dep.path("thirdparty/opnmidi"));

        compile.linkLibrary(opnmidi);
    }

    {
        const timidity = b.addStaticLibrary(.{
            .name = "opnmidi",
            .target = target,
            .optimize = optimize,
        });

        timidity.addCSourceFiles(.{
            .root = dep.path("thirdparty/timidity"),
            .flags = &(fast_math ++ stricmp ++ [_][]const u8{
                "-I",
                ".",
                "-I",
                "timidity",
            }),
            .files = &[_][]const u8{
                "common.cpp",
                "instrum.cpp",
                "instrum_dls.cpp",
                "instrum_font.cpp",
                "instrum_sf2.cpp",
                "mix.cpp",
                "playmidi.cpp",
                "resample.cpp",
                "timidity.cpp",
            },
        });

        timidity.linkLibC();
        timidity.linkLibCpp();
        timidity.addIncludePath(dep.path("thirdparty/timidity"));
        timidity.addIncludePath(dep.path("thirdparty/timidity/timidity"));

        compile.linkLibrary(timidity);
    }

    {
        const timiditypp = b.addStaticLibrary(.{
            .name = "timiditypp",
            .target = target,
            .optimize = optimize,
        });

        timiditypp.addCSourceFiles(.{
            .root = dep.path("thirdparty/timidityplus"),
            .flags = &fast_math,
            .files = &[_][]const u8{
                "fft4g.cpp",
                "reverb.cpp",
                "common.cpp",
                "configfile.cpp",
                "effect.cpp",
                "filter.cpp",
                "freq.cpp",
                "instrum.cpp",
                "mblock.cpp",
                "mix.cpp",
                "playmidi.cpp",
                "quantity.cpp",
                "readmidic.cpp",
                "recache.cpp",
                "resample.cpp",
                "sbkconv.cpp",
                "sffile.cpp",
                "sfitem.cpp",
                "smplfile.cpp",
                "sndfont.cpp",
                "tables.cpp",
            },
        });

        timiditypp.linkLibC();
        timiditypp.linkLibCpp();
        timiditypp.addIncludePath(dep.path("thirdparty/timidityplus"));
        timiditypp.addIncludePath(dep.path("thirdparty/timidityplus/timiditypp"));

        compile.linkLibrary(timiditypp);
    }

    {
        const wildmidi = b.addStaticLibrary(.{
            .name = "wildmidi",
            .target = target,
            .optimize = optimize,
        });

        wildmidi.addCSourceFiles(.{
            .root = dep.path("thirdparty/wildmidi"),
            .flags = &(fast_math ++ stricmp ++ strnicmp ++ [_][]const u8{
                "-fomit-frame-pointer",
            }),
            .files = &[_][]const u8{
                "file_io.cpp",
                "gus_pat.cpp",
                "reverb.cpp",
                "wildmidi_lib.cpp",
                "wm_error.cpp",
            },
        });

        wildmidi.linkLibC();
        wildmidi.linkLibCpp();
        wildmidi.addIncludePath(dep.path("thirdparty/wildmidi"));
        wildmidi.addIncludePath(dep.path("thirdparty/wildmidi/wildmidi"));

        compile.linkLibrary(wildmidi);
    }

    compile.linkSystemLibrary2("flac", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });
    compile.linkSystemLibrary2("sndfile", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });
    compile.linkSystemLibrary2("ogg", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });
    compile.linkSystemLibrary2("opus", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });
    compile.linkSystemLibrary2("vorbis", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });
    compile.linkSystemLibrary2("vorbisenc", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });

    compile.linkLibrary(lib);
    compile.addSystemIncludePath(dep.path("include"));
}
