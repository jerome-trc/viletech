//! Types and functions relating to the [DMX MUS](https://doomwiki.org/wiki/MUS) file format.

const std = @import("std");

pub const Result = union(enum) {
    ok,

    invalid_controller_num: struct { pos: u64, num: u8 },
    magic_number,
    no_data: struct { len: usize, score_start: u16 },
    /// The MUS data is not even large enough to fit a header (14 bytes).
    undersize,
    unexpected_eoi,
    unknown_event: struct { pos: u64, desc: u8 },
    write,
};

/// From SLADE's port of mus2midi. See `/legal/slade.txt`.
pub fn toMidi(bytes: []const u8, writer: anytype) Result {
    const WriterT = @TypeOf(writer);

    const MusHeader = extern struct {
        id: [4]u8,
        score_len: u16,
        score_start: u16,
        channels_1: u16,
        channels_2: u16,
        instrument_count: u16,
    };

    const Channels = struct {
        const undef: i16 = -1;
        const mus_percussion: i16 = 15;
        const midi_percussion: i16 = 9;

        mapping: [16]i16 = [_]i16{@This().undef} ** 16,
        velocities: [16]u8 = [_]u8{127} ** 16,

        fn allocate(self: *const @This()) i16 {
            var ret = std.mem.max(i16, &self.mapping);

            if (ret == undef) ret = undef + 1;

            ret += 1;

            // Don't allocate the MIDI percussion channel.
            if (ret == midi_percussion) ret += 1;

            return ret;
        }

        fn getOrAllocate(self: *@This(), channel: u8) u8 {
            if (channel == mus_percussion) return midi_percussion;

            if (self.mapping[channel] == undef)
                self.mapping[channel] = self.allocate();

            return std.math.lossyCast(u8, self.mapping[channel]);
        }
    };

    const controller_map = comptime [15]u8{
        0x00, 0x20, 0x01, 0x07, 0x0A,
        0x0B, 0x5B, 0x5D, 0x40, 0x43,
        0x78, 0x7B, 0x7E, 0x7F, 0x79,
    };

    if (bytes.len < @sizeOf(MusHeader)) {
        return Result{ .undersize = {} };
    }

    var channels = Channels{};

    const header = std.mem.littleToNative(
        MusHeader,
        std.mem.bytesAsValue(MusHeader, bytes[0..@sizeOf(MusHeader)]).*,
    );

    if (!std.mem.eql(u8, &header.id, &[_]u8{ 'M', 'U', 'S', 0x1a })) {
        return Result{ .magic_number = {} };
    }

    var cursor = std.io.fixedBufferStream(bytes);
    cursor.seekTo(header.score_start) catch unreachable;

    if ((writer.write(
        midi_header[0..],
    ) catch return Result{ .write = {} }) != @sizeOf(@TypeOf(midi_header))) {
        return Result{ .write = {} };
    }

    var ctx = Context{
        .track_len = 0,
        .delta = 0,
    };

    var at_score_end = false;

    while (!at_score_end) {
        // Handle a block of events.
        while (!at_score_end) {
            const edesc = cursor.reader().readByte() catch
                return Result{ .unexpected_eoi = {} };
            const channel = channels.getOrAllocate(edesc & 0x0f);

            switch (edesc & 0x70) {
                0 => { // Key release
                    const key = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };

                    switch (ctx.writeReleaseKey(channel, key, writer)) {
                        .ok => {},
                        else => |r| return r,
                    }
                },
                16 => { // Key press
                    const key = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };

                    if ((key & 0x80) != 0) {
                        const vel = cursor.reader().readByte() catch
                            return Result{ .unexpected_eoi = {} };
                        channels.velocities[channel] = vel & 0x7f;
                    }

                    switch (ctx.writePressKey(
                        channel,
                        key,
                        channels.velocities[channel],
                        writer,
                    )) {
                        .ok => {},
                        else => |r| return r,
                    }
                },
                32 => { // Pitch wheel
                    const key = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };

                    switch (ctx.writePitchWheel(channel, @as(i16, key) * 64, writer)) {
                        .ok => {},
                        else => |r| return r,
                    }
                },
                48 => { // System event
                    const ctrl_num = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };

                    if ((ctrl_num < 10) or (ctrl_num > 14)) {
                        return Result{ .invalid_controller_num = .{
                            .pos = cursor.getPos() catch unreachable,
                            .num = ctrl_num,
                        } };
                    }

                    switch (ctx.writeControllerChange(
                        channel,
                        controller_map[ctrl_num],
                        0,
                        writer,
                    )) {
                        .ok => {},
                        else => |r| return r,
                    }
                },
                64 => { // Change controller
                    const ctrl_num = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };
                    const ctrl_val = cursor.reader().readByte() catch
                        return Result{ .unexpected_eoi = {} };

                    if (ctrl_num == 0) {
                        switch (ctx.writePatchChange(channel, ctrl_val, writer)) {
                            .ok => {},
                            else => |r| return r,
                        }
                    } else {
                        if ((ctrl_num < 1) or (ctrl_num > 9)) {
                            return Result{ .invalid_controller_num = .{
                                .pos = cursor.getPos() catch unreachable,
                                .num = ctrl_num,
                            } };
                        }

                        switch (ctx.writeControllerChange(
                            channel,
                            controller_map[ctrl_num],
                            ctrl_val,
                            writer,
                        )) {
                            .ok => {},
                            else => |r| return r,
                        }
                    }
                },
                96 => at_score_end = true,
                else => |other| {
                    return Result{ .unknown_event = .{
                        .pos = cursor.getPos() catch unreachable,
                        .desc = other,
                    } };
                },
            }

            if ((edesc & 0x80) != 0) {
                break;
            }
        }

        // Now read the time code...
        if (!at_score_end) {
            ctx.delta = 0;

            while (true) {
                const working = cursor.reader().readByte() catch
                    return Result{ .unexpected_eoi = {} };

                ctx.delta = ctx.delta * 128 + (working & 0x7f);

                if ((working & 0x80) == 0) {
                    break;
                }
            }
        }
    }

    switch (ctx.writeTrackEnd(writer)) {
        .ok => {},
        else => |r| return r,
    }

    const track_len = [_]u8{
        @truncate((ctx.track_len >> 24) & 0x00_00_00_ff),
        @truncate((ctx.track_len >> 16) & 0x00_00_00_ff),
        @truncate((ctx.track_len >> 8) & 0x00_00_00_ff),
        @truncate(ctx.track_len & 0x00_00_00_ff),
    };

    if (@hasDecl(WriterT, "seekTo")) {
        writer.seekTo(18) catch return Result{ .write = {} };

        const w = writer.write(&track_len) catch return Result{ .write = {} };
        if (w != track_len.len) return Result{ .write = {} };
    } else if (@hasField(WriterT, "context")) {
        var seeker = std.io.fixedBufferStream(writer.context.self.items);
        seeker.seekTo(18) catch return Result{ .write = {} };

        const w = seeker.write(&track_len) catch return Result{ .write = {} };
        if (w != track_len.len) return Result{ .write = {} };
    } else @compileError("`writer` must support seeking");

    return Result{ .ok = {} };
}

// Details /////////////////////////////////////////////////////////////////////

const midi_header = [_]u8{
    'M', 'T', 'h', 'd', // Main header
    0x00, 0x00, 0x00, 0x06, // Header size
    0x00, 0x00, // MIDI type (0)
    0x00, 0x01, // Number of tracks
    0x00, 0x46, // Resolution
    'M', 'T', 'r', 'k', // Start of track
    0x00, 0x00, 0x00, 0x00, // Placeholder for track length
};

const channel_velocities = [_]u8{
    127, 127, 127, 127, 127, 127, 127, 127,
    127, 127, 127, 127, 127, 127, 127, 127,
};

const Context = struct {
    track_len: u32,
    delta: u32,

    fn writeTime(self: *Context, writer: anytype) Result {
        var time = self.delta;
        var buf = time & 0x7f;

        while (true) {
            time >>= 7;

            if (time == 0) break;

            buf <<= 8;
            buf |= (time & 0x7f) | 0x80;
        }

        while (true) {
            const write_val = @as(u8, @truncate(buf)) & 0xff;
            writer.writeByte(write_val) catch return Result{ .write = {} };
            self.track_len += 1;

            if ((buf & 0x80) != 0) {
                buf >>= 8;
            } else {
                self.delta = 0;
                break;
            }
        }

        return Result{ .ok = {} };
    }

    fn writeControllerChange(
        self: *Context,
        channel: u8,
        control: u8,
        value: u8,
        writer: anytype,
    ) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const bytes = if ((value & 0x80) != 0)
            [_]u8{ 176 | channel, control & 0x7f, 0x7f }
        else
            [_]u8{ 176 | channel, control & 0x7f, value };

        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 3;
        return Result{ .ok = {} };
    }

    fn writePatchChange(
        self: *Context,
        channel: u8,
        patch: u8,
        writer: anytype,
    ) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const bytes = [_]u8{ 192 | channel, patch & 0x7f };
        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 2;
        return Result{ .ok = {} };
    }

    fn writePitchWheel(
        self: *Context,
        channel: u8,
        wheel: i16,
        writer: anytype,
    ) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const wheel8 = std.math.lossyCast(u8, wheel);
        const wheel_shr7_8: u8 = std.math.lossyCast(u8, wheel >> 7);
        const bytes = [_]u8{ 224 | channel, wheel8 & 0x7f, wheel_shr7_8 & 0x7f };
        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 3;
        return Result{ .ok = {} };
    }

    fn writePressKey(
        self: *Context,
        channel: u8,
        key: u8,
        velocity: u8,
        writer: anytype,
    ) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const bytes = [_]u8{ 144 | channel, key & 0x7f, velocity & 0x7f };
        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 3;
        return Result{ .ok = {} };
    }

    fn writeReleaseKey(self: *Context, channel: u8, key: u8, writer: anytype) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const bytes = [_]u8{ 128 | channel, key & 0x7f, 0 };
        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 3;
        return Result{ .ok = {} };
    }

    fn writeTrackEnd(self: *Context, writer: anytype) Result {
        switch (self.writeTime(writer)) {
            .ok => {},
            else => |r| return r,
        }

        const bytes = [_]u8{ 0xff, 0x2f, 0x00 };
        const w = writer.write(&bytes) catch return Result{ .write = {} };
        if (w != bytes.len) return Result{ .write = {} };

        self.track_len += 3;
        return Result{ .ok = {} };
    }
};
