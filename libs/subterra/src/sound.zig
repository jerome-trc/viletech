//! See <https://doomwiki.org/wiki/Sound>.

const builtin = @import("builtin");
const std = @import("std");

const Self = @This();

pub const Error = error{
    /// Neither 0 nor 3.
    InvalidFormatNumber,
    Misaligned,
    /// The given "lump" is not even large enough to fit the expected header.
    HeaderTruncated,
    /// The given "lump" is not even large enough to fit a format number.
    UndersizeFile,
    /// The given "lump" is not even large enough to fit as many samples
    /// as the header declares the sound to contain.
    SamplesTruncated,
};

pub const Dmx = struct {
    pub const Header = extern struct {
        /// This is expected to be 3 to distinguish from a speaker sound.
        format: u16,
        sample_rate: u16,
        num_samples: u32,
        pad: [16]u8,
    };

    /// This field is always in native endianness.
    header: Header,
    samples: []const u8,
};

pub const Speaker = struct {
    pub const Header = extern struct {
        /// This is expected to be 0 to distinguish from a DMX sound.
        format: u16,
        num_samples: u16,
    };

    /// This field is always in native endianness.
    header: Header,
    samples: []const u8,
};

pub const ReadOut = union(enum) {
    dmx: Dmx,
    speaker: Speaker,
};

/// For use when you don't know whether the given bytes comprise a DMX sound
/// or a speaker sound. If they appear to be the former, this function will
/// attempt to realign `bytes` to 4 bytes, and return an error if it fails.
pub fn read(bytes: []align(2) const u8) Error!ReadOut {
    if (bytes.len < @sizeOf(Speaker.Header)) return error.UndersizeFile;
    const fmt_no = std.mem.readInt(u16, bytes[0..2], .little);

    switch (fmt_no) {
        0 => {
            var header = std.mem.bytesAsValue(Speaker.Header, bytes[0..@sizeOf(Speaker.Header)]).*;

            if (builtin.cpu.arch.endian() != .little)
                std.mem.byteSwapAllFields(Speaker.Header, &header);

            const samples_start = @sizeOf(Speaker.Header);
            const samples_end = samples_start + header.num_samples;

            if (bytes.len < samples_end) return error.SamplesTruncated;

            return ReadOut{ .speaker = Speaker{
                .header = header,
                .samples = bytes[samples_start..samples_end],
            } };
        },
        3 => {
            if (!std.mem.isAligned(@intFromPtr(bytes.ptr), @alignOf(Dmx.Header)))
                return error.Misaligned;

            if (bytes.len < @sizeOf(Dmx.Header)) return error.UndersizeFile;
            var header = std.mem.bytesAsValue(Dmx.Header, bytes[0..@sizeOf(Dmx.Header)]).*;

            if (builtin.cpu.arch.endian() != .little)
                std.mem.byteSwapAllFields(Dmx.Header, &header);

            const samples_start = @sizeOf(Dmx.Header);
            const samples_end = samples_start + (header.num_samples -| 16);

            if (bytes.len < samples_end) return error.SamplesTruncated;

            return ReadOut{ .dmx = Dmx{
                .header = header,
                .samples = bytes[samples_start..samples_end],
            } };
        },
        else => return error.InvalidFormatNumber,
    }
}

test "DMX, smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom/DSITEMUP.lmp", .{});
    defer f.close();

    const bytes = try f.readToEndAllocOptions(
        std.testing.allocator,
        1024 * 3,
        null,
        @alignOf(Dmx.Header),
        null,
    );
    defer std.testing.allocator.free(bytes);

    const sound = try read(bytes);
    const dmxsnd = sound.dmx;

    try std.testing.expectEqual(3, dmxsnd.header.format);
    try std.testing.expectEqual(11025, dmxsnd.header.sample_rate);
    try std.testing.expectEqual(2205, dmxsnd.header.num_samples);
    try std.testing.expectEqual(2189, dmxsnd.samples.len);
}

test "Speaker, smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom/DPITEMUP.lmp", .{});
    defer f.close();

    const bytes = try f.readToEndAllocOptions(
        std.testing.allocator,
        128,
        null,
        @alignOf(Speaker.Header),
        null,
    );
    defer std.testing.allocator.free(bytes);

    const sound = try read(bytes);
    const spsnd = sound.speaker;

    try std.testing.expectEqual(0, spsnd.header.format);
    try std.testing.expectEqual(83, spsnd.header.num_samples);
    try std.testing.expectEqual(83, spsnd.samples.len);
}
