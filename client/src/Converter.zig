//! The file format conversion feature of the CLI.

const std = @import("std");
const BufReader = std.io.BufferedReader(4096, std.fs.File.Reader);
const BufWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

const ContentId = @import("viletech").ContentId;
const subterra = @import("subterra");

const Self = @This();

const full_help_text = blk: {
    var str: []const u8 = "Known file formats (names are case-insensitive):";

    for (std.enums.values(ContentId)) |c_id| {
        if (c_id == ContentId.unknown) continue;
        str = std.fmt.comptimePrint(str ++ "\n    {s}", .{@tagName(c_id)})[0..];
    }

    str = (str ++ "\n\nSupported conversions:")[0..];

    for (conversions) |conv| {
        str = std.fmt.comptimePrint(str ++ "\n    {s} -> {s}", .{
            conv.from.prettyName(),
            conv.to.prettyName(),
        })[0..];
    }

    break :blk str;
};

pub const Args = struct {
    in: []const u8 = "",
    out: []const u8 = "",
    @"in-format": []const u8 = "",
    @"out-format": []const u8 = "",

    pub const shorthands = .{
        .i = "in",
        .o = "out",
        .I = "in-format",
        .O = "out-format",
    };

    pub const meta = .{
        .full_text = full_help_text,
        .usage_summary = "conv [options...]",
        .option_docs = .{
            .in = "Input file (- for stdin)",
            .out = "Output file (- for stdout)",
            .@"in-format" = "Input file format (omit to try heuristics)",
            .@"out-format" = "Output file format (omit to try heuristics)",
        },
    };
};

arena: std.heap.ArenaAllocator,

stderr_file: std.fs.File.Writer,
stderr_bw: BufWriter,

input: std.fs.File.Reader,
output: BufWriter,

fn eprintln(
    self: *Self,
    comptime format: []const u8,
    args: anytype,
) BufWriter.Error!void {
    try self.stderr_bw.writer().print(format ++ "\n", args);
    try self.stderr_bw.flush();
}

pub fn run(args: Args) !void {
    // TODO:
    // - only create output file at last second
    // `--force|-f` to permit overwriting file at output path
    // - specify error return type.

    const stderr_file = std.io.getStdErr().writer();

    var self = Self{
        .arena = std.heap.ArenaAllocator.init(std.heap.c_allocator),
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .input = undefined,
        .output = undefined,
    };

    defer _ = self.arena.reset(.free_all);

    if (args.in.len == 0) {
        try self.eprintln("No input file given", .{});
        return error.NoInput;
    }

    if (args.out.len == 0) {
        try self.eprintln("No output file given", .{});
        return error.NoOutput;
    }

    var in_format = ContentId.unknown;
    var out_format = ContentId.unknown;

    var in_file: std.fs.File = undefined;
    var close_in_file = false;
    var out_file: std.fs.File = undefined;
    var close_out_file = false;

    if (std.mem.eql(u8, args.in, "-")) {
        in_file = std.io.getStdIn();
        self.input = in_file.reader();
    } else {
        var path = args.in;

        if (!std.fs.path.isAbsolute(path)) {
            path = try std.fs.cwd().realpathAlloc(self.arena.allocator(), args.in);
        }

        if (ContentId.by_filestem.get(std.fs.path.basename(path))) |c_id| {
            in_format = c_id;
        } else if (ContentId.by_extension.get(std.fs.path.extension(path))) |c_id| {
            in_format = c_id;
        }

        in_file = try std.fs.openFileAbsolute(path, .{});
        self.input = in_file.reader();
        close_in_file = true;
    }

    if (std.mem.eql(u8, args.out, "-")) {
        out_file = std.io.getStdOut();
        self.output = std.io.bufferedWriter(out_file.writer());
    } else {
        var path = args.out;

        if (!std.fs.path.isAbsolute(path)) {
            path = try std.fs.cwd().realpathAlloc(self.arena.allocator(), args.out);
        }

        if (ContentId.by_filestem.get(std.fs.path.basename(path))) |c_id| {
            out_format = c_id;
        } else if (ContentId.by_extension.get(std.fs.path.extension(path))) |c_id| {
            out_format = c_id;
        }

        out_file = try std.fs.createFileAbsolute(path, .{ .truncate = true });
        self.output = std.io.bufferedWriter(out_file.writer());
        close_out_file = true;
    }

    defer {
        if (close_in_file) in_file.close();
        if (close_out_file) out_file.close();
    }
    // errdefer if (new_out_file) @compileError("TODO");

    var unknown_formats = false;

    for (std.enums.values(ContentId)) |c_id| {
        if (std.ascii.eqlIgnoreCase(args.@"in-format", @tagName(c_id))) {
            in_format = c_id;
        }

        if (std.ascii.eqlIgnoreCase(args.@"out-format", @tagName(c_id))) {
            out_format = c_id;
        }
    }

    // TODO: aliases e.g. "mus" for "dmx_mus"?

    if (in_format == ContentId.unknown) {
        try self.eprintln("Input format not given and could not be guessed.", .{});
        unknown_formats = true;
    }

    if (out_format == ContentId.unknown) {
        try self.eprintln("Output format not given and could not be guessed.", .{});
        unknown_formats = true;
    }

    if (unknown_formats) {
        return error.UnknownFormats;
    }

    inline for (conversions) |conv| {
        if (in_format == conv.from and out_format == conv.to) {
            try self.eprintln(
                \\Attempting conversion...
                \\    from: {s}
                \\    to:   {s}
            , .{ in_format.prettyName(), out_format.prettyName() });
            try conv.func(&self);
            return;
        }
    }

    try self.eprintln(
        "Unsupported conversion: {s} -> {s}",
        .{
            in_format.prettyName(),
            out_format.prettyName(),
        },
    );
    return error.Unsupported;
}

const Conversion = struct {
    from: ContentId,
    to: ContentId,
    func: fn (self: *Self) anyerror!void,
};

const conversions = [_]Conversion{
    .{ .from = ContentId.dmx_mus, .to = ContentId.midi, .func = musToMidi },
};

fn musToMidi(self: *Self) anyerror!void {
    const bytes = try self.input.readAllAlloc(self.arena.allocator(), 1024 * 1024);

    var intermediate = std.ArrayListUnmanaged(u8){};

    switch (subterra.mus.toMidi(bytes, intermediate.writer(self.arena.allocator()))) {
        .ok => {
            const w = try self.output.write(intermediate.items);

            if (w != intermediate.items.len)
                try self.eprintln(
                    "MUS conversion failed to write all {} bytes.",
                    .{intermediate.items.len},
                )
            else
                try self.eprintln("MUS conversion successful.", .{});
        },
        .undersize => {
            try self.eprintln("Input is not even large enough to fit a header.", .{});
        },
        .unexpected_eoi => {
            try self.eprintln("Unexpected end-of-input.", .{});
        },
        .write => {
            try self.eprintln("Failed to write output.", .{});
        },
        else => {},
    }
}

pub const Error = error{
    NoInput,
    NoOutput,
    UnknownFormats,
    Unsupported,
} || BufWriter.Error;
