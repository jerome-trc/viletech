const builtin = @import("builtin");
const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
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
        "depend/zmsx/source",
        "-I",
        "depend/zmsx/source/zmsx",
        "-I",
        "depend/zmsx/include",
        "-isystem",
        "depend/zmsx/thirdparty/adlmidi",
        "-isystem",
        "depend/zmsx/thirdparty/game-music-emu",
        "-isystem",
        "depend/zmsx/thirdparty/miniz",
        "-isystem",
        "depend/zmsx/thirdparty/oplsynth",
        "-isystem",
        "depend/zmsx/thirdparty/opnmidi",
        "-isystem",
        "depend/zmsx/thirdparty/timidity",
        "-isystem",
        "depend/zmsx/thirdparty/timidityplus",
        "-isystem",
        "depend/zmsx/thirdparty/wildmidi",
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
        .root = b.path("depend/zmsx/source"),
        .flags = src_flags,
        .files = &[_][]const u8{
            "loader/i_module.cpp",
            "mididevices/music_base_mididevice.cpp",
            "mididevices/music_adlmidi_mididevice.cpp",
            "mididevices/music_opl_mididevice.cpp",
            "mididevices/music_opnmidi_mididevice.cpp",
            "mididevices/music_timiditypp_mididevice.cpp",
            "mididevices/music_fluidsynth_mididevice.cpp",
            "mididevices/music_softsynth_mididevice.cpp",
            "mididevices/music_timidity_mididevice.cpp",
            "mididevices/music_wildmidi_mididevice.cpp",
            "mididevices/music_wavewriter_mididevice.cpp",
            "midisources/midisource.cpp",
            "midisources/midisource_mus.cpp",
            "midisources/midisource_smf.cpp",
            "midisources/midisource_hmi.cpp",
            "midisources/midisource_xmi.cpp",
            "midisources/midisource_mids.cpp",
            "streamsources/music_dumb.cpp",
            "streamsources/music_gme.cpp",
            "streamsources/music_libsndfile.cpp",
            "streamsources/music_opl.cpp",
            "streamsources/music_xa.cpp",
            "musicformats/music_stream.cpp",
            "musicformats/music_midi.cpp",
            "musicformats/music_cd.cpp",
            "decoder/sounddecoder.cpp",
            "decoder/sndfile_decoder.cpp",
            "decoder/mpg123_decoder.cpp",
            "zmsx/configuration.cpp",
            "zmsx/zmsx.cpp",
            "zmsx/critsec.cpp",
            "loader/test.c",
        },
    });

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/adlmidi"),
        .flags = &(fast_math ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/adlmidi",
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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/dumb"),
        .flags = &(fast_math ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/dumb/include",
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

    fluidsynth(b, compile, lib);

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/game-music-emu/gme"),
        .flags = &(fast_math ++ [_][]const u8{
            "-fomit-frame-pointer",
            "-fwrapv",
            "-I",
            "depend/zmsx/thirdparty/game-music-emu/gme",
            "-isystem",
            "depend/zmsx/thirdparty/miniz",
            "-DHAVE_ZLIB_H",
        }),
        .files = &[_][]const u8{
            "Blip_Buffer.cpp",
            "Classic_Emu.cpp",
            "Data_Reader.cpp",
            "Dual_Resampler.cpp",
            "Effects_Buffer.cpp",
            "Fir_Resampler.cpp",
            "gme.cpp",
            "Gme_File.cpp",
            "M3u_Playlist.cpp",
            "Multi_Buffer.cpp",
            "Music_Emu.cpp",
            // TODO: optional emulators?
        },
    });

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/miniz"),
        .flags = &[_][]const u8{ "-I", "depend/zmsx/thirdparty/miniz" },
        .files = &[_][]const u8{"miniz.c"},
    });

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/oplsynth"),
        .flags = &(fast_math ++ stricmp ++ strnicmp ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/oplsynth",
            "-I",
            "depend/zmsx/thirdparty/oplsynth/oplsynth",
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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/opnmidi"),
        .flags = &(fast_math ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/opnmidi",
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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/timidity"),
        .flags = &(fast_math ++ stricmp ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/timidity",
            "-I",
            "depend/zmsx/thirdparty/timidity/timidity",
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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/timidityplus"),
        .flags = &(fast_math ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/timidityplus",
            "-I",
            "depend/zmsx/thirdparty/timidityplus/timiditypp",
        }),
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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/wildmidi"),
        .flags = &(fast_math ++ stricmp ++ strnicmp ++ [_][]const u8{
            "-I",
            "depend/zmsx/thirdparty/wildmidi",
            "-I",
            "depend/zmsx/thirdparty/wildmidi/wildmidi",
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

    compile.linkSystemLibrary2("sndfile", .{
        .needed = true,
        .preferred_link_mode = .static,
        .use_pkg_config = .yes,
    });

    compile.linkLibrary(lib);
    compile.addSystemIncludePath(b.path("depend/zmsx/include"));
}

fn fluidsynth(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    lib: *std.Build.Step.Compile,
) void {
    var fluidsynth_flags: []const []const u8 = &[_][]const u8{
        "-I",
        "depend/zmsx/source/decoder",
        "-I",
        "depend/zmsx/thirdparty",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/include",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/drivers",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/synth",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/rvoice",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/midi",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/utils",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/sfloader",
        "-I",
        "depend/zmsx/thirdparty/fluidsynth/src/bindings",
    };

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

    lib.addCSourceFiles(.{
        .root = b.path("depend/zmsx/thirdparty/fluidsynth/src"),
        .flags = fluidsynth_flags,
        .files = &[_][]const u8{
            "utils/fluid_conv.c",
            "utils/fluid_hash.c",
            "utils/fluid_list.c",
            "utils/fluid_ringbuffer.c",
            "utils/fluid_settings.c",
            "utils/fluid_sys.c",
            "sfloader/fluid_defsfont.c",
            "sfloader/fluid_sfont.c",
            "sfloader/fluid_sffile.c",
            "sfloader/fluid_samplecache.c",
            "rvoice/fluid_adsr_env.c",
            "rvoice/fluid_chorus.c",
            "rvoice/fluid_iir_filter.c",
            "rvoice/fluid_lfo.c",
            "rvoice/fluid_rvoice.c",
            "rvoice/fluid_rvoice_dsp.c",
            "rvoice/fluid_rvoice_event.c",
            "rvoice/fluid_rvoice_mixer.c",
            "rvoice/fluid_rev.c",
            "synth/fluid_chan.c",
            "synth/fluid_event.c",
            "synth/fluid_gen.c",
            "synth/fluid_mod.c",
            "synth/fluid_synth.c",
            "synth/fluid_synth_monopoly.c",
            "synth/fluid_tuning.c",
            "synth/fluid_voice.c",
            "midi/fluid_midi.c",
            "midi/fluid_midi_router.c",
            "midi/fluid_seqbind.c",
            "midi/fluid_seqbind_notes.cpp",
            "midi/fluid_seq.c",
            "midi/fluid_seq_queue.cpp",
            "drivers/fluid_adriver.c",
            "drivers/fluid_mdriver.c",
            "bindings/fluid_filerenderer.c",
            "bindings/fluid_ladspa.c",
        },
    });

    if (builtin.os.tag != .windows) {
        compile.linkSystemLibrary2("glib-2.0", .{
            .needed = true,
            .preferred_link_mode = .static,
            .use_pkg_config = .yes,
        });
    }
}
