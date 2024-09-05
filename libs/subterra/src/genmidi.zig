//! See <https://doomwiki.org/wiki/GENMIDI>.

const std = @import("std");

pub const GenMidi = extern struct {
    /// Expected to contain `#OPL_II#`.
    magic: [8]u8,
    instruments: [175]Instrument,
    instrument_names: [175][32]u8,

    /// The lifetime of the returned pointer is tied to that of `bytes`.
    pub fn read(bytes: []align(@alignOf(GenMidi)) const u8) Error!*const GenMidi {
        if (bytes.len != @sizeOf(GenMidi)) return error.WrongFileSize;
        if (!std.mem.eql(u8, bytes[0..8], "#OPL_II#")) return error.WrongMagicNumber;

        return std.mem.bytesAsValue(GenMidi, bytes);
    }
};

pub const Instrument = extern struct {
    pub const Flags = packed struct(u16) {
        fixed_pitch: bool,
        instrument_65: bool,
        double_voice: bool,

        _pad: u13,
    };

    pub const Voice = extern struct {
        pub const Data = extern struct {
            multi: i8,
            /// a.k.a. "attack".
            decay: i8,
            /// a.k.a. "sustain".
            release: i8,
            waveform: i8,
            key_scale: i8,
            output: i8,
        };

        modulator: Data,
        feedback: i8,
        carrier: Data,
        _unused: i8,
        base_note_offs: u16,
    };

    _flags: u16,
    fine_tuning: i8,
    fixed_note: u8,
    voices: [2]Voice,

    pub fn flags(self: *const Instrument) Flags {
        return @bitCast(std.mem.littleToNative(u16, self._flags));
    }
};

pub const Error = error{
    /// GENMIDI "lump" is not large enough to fit the expected data.
    WrongFileSize,
    WrongMagicNumber,
};

test "GENMIDI, smoke" {
    const cfg = @import("cfg");

    if (cfg.genmidi_sample.len == 0) return error.SkipZigTest;

    const f = try std.fs.cwd().openFile(cfg.genmidi_sample, .{});
    defer f.close();

    const bytes = try f.readToEndAllocOptions(
        std.testing.allocator,
        1024 * 12,
        null,
        @alignOf(GenMidi),
        null,
    );
    defer std.testing.allocator.free(bytes);

    const genmidi = try GenMidi.read(bytes);

    try std.testing.expectEqualStrings("#OPL_II#", genmidi.magic[0..]);
    try std.testing.expectEqual(0, genmidi.instruments[0]._flags);
    try std.testing.expectEqual(-128, genmidi.instruments[0].fine_tuning);
    try std.testing.expectEqual(0, genmidi.instruments[0].fixed_note);

    try std.testing.expectEqual(0, genmidi.instruments[0].voices[0].base_note_offs);
    try std.testing.expectEqual(10, genmidi.instruments[0].voices[0].feedback);

    try std.testing.expectEqual(
        Instrument.Voice.Data{
            .decay = -16,
            .key_scale = 64,
            .multi = 48,
            .release = -13,
            .output = 20,
            .waveform = 1,
        },
        genmidi.instruments[0].voices[0].modulator,
    );
    try std.testing.expectEqual(
        Instrument.Voice.Data{
            .decay = -15,
            .key_scale = 0,
            .multi = 48,
            .release = -12,
            .output = 0,
            .waveform = 1,
        },
        genmidi.instruments[0].voices[0].carrier,
    );
}
