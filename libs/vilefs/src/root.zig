//! # VileFS
//!
//! VileTech's custom virtual file system, used by the editor and CLI tools
//! which require such a facility to function.

const std = @import("std");

const wadload = @import("wadload");
const wad = wadload.wad;

/// Type alias to increase clarity.
pub const VirtualPath = []const u8;
/// Type alias to increase clarity.
pub const RealPath = []const u8;

/// See `VirtualFs`.
pub const FsConfig = struct {
    stem8_map: bool = false,
};

pub fn VirtualFs(config: FsConfig) type {
    return struct {
        const Self = @This();

        alloc: struct {
            metadata: std.mem.Allocator,
            content: std.mem.Allocator,
        },
        max_read_size: usize,

        root: Folder,
        total_entries: usize,
        total_folders: usize,

        stem8_map: if (config.stem8_map)
            Stem8MapUnmanaged(*const Entry)
        else
            void,

        /// The caller guarantees that pointers coming from either allocator
        /// will remaing stable regardless of how many allocations/de-allocations
        /// are performed.
        pub fn init(
            metadata_allocator: std.mem.Allocator,
            content_allocator: std.mem.Allocator,
            max_read_size: usize,
        ) Self {
            return Self{
                .alloc = .{
                    .metadata = metadata_allocator,
                    .content = content_allocator,
                },
                .max_read_size = max_read_size,

                .root = Folder{
                    .name = "/",
                    .parent = null,
                    .entries = .{},
                    .subfolders = .{},
                },
                .total_entries = 0,
                .total_folders = 1,

                .stem8_map = if (config.stem8_map) .{} else {},
            };
        }

        pub fn deinit(self: *Self) void {
            self.clearRecur(&self.root);
            self.root.subfolders.deinit(self.alloc.metadata);
            self.root.entries.deinit(self.alloc.metadata);

            if (config.stem8_map) {
                self.stem8_map.deinit(self.alloc.metadata);
            }
        }

        pub fn clear(self: *Self) void {
            self.clearRecur(&self.root);
            self.total_entries = 0;
            self.total_folders = 1;

            if (config.stem8_map) {
                self.stem8_map.clearRetainingCapacity();
            }
        }

        fn clearRecur(self: *Self, folder: *Folder) void {
            for (folder.entries.items) |entry| {
                self.freeEntry(entry);
            }

            for (folder.subfolders.items) |subfolder| {
                self.clearRecur(subfolder);
                self.alloc.metadata.destroy(subfolder);
            }

            if (folder != &self.root) {
                self.alloc.metadata.free(folder.name);
                folder.subfolders.deinit(self.alloc.metadata);
                folder.entries.deinit(self.alloc.metadata);
            } else {
                folder.subfolders.clearRetainingCapacity();
                folder.entries.clearRetainingCapacity();
            }
        }

        /// After this returns, `entry` points to freed memory.
        /// The parent folder of the returned entry is valid but always empty.
        pub fn deleteEntry(self: *Self, entry: *const Entry) Entry {
            const pos = for (entry.parent.entries, 0..) |*e, i| blk: {
                if (e == entry) break :blk i;
            } else unreachable;

            entry.parent.entries.orderedRemove(pos);
            self.total_entries -= 1;

            if (config.stem8_map) {
                std.debug.assert(self.stem8_map.remove(entry.stem8()));
            }

            var ret = entry.*;
            ret.parent = &nil_folder;
            self.freeEntry(entry);
            return ret;
        }

        pub fn mount(self: *Self, path: RealPath) anyerror!void {
            if (!std.fs.path.isAbsolute(path)) {
                return error.RelativePath;
            }

            if (std.fs.openDirAbsolute(path, .{})) |dir| {
                var d = dir;
                defer d.close();
                try self.mountDir(&self.root, NamedDir{ .name = path, .inner = &d });
                return;
            } else |err| {
                switch (err) {
                    error.NotDir => {},
                    else => |e| return e,
                }
            }

            var file = try std.fs.openFileAbsolute(path, .{});
            defer file.close();

            try self.dispatchMount(&self.root, NamedFile{
                .name = std.fs.path.basename(path),
                .inner = &file,
            });
        }

        fn dispatchMount(self: *Self, parent: *Folder, file: NamedFile) anyerror!void {
            var magic_buf: [24]u8 = undefined;
            const bytes_read = try file.inner.read(magic_buf[0..]);
            const magic = magic_buf[0..bytes_read];
            try file.inner.seekTo(0);

            if (std.mem.eql(u8, magic[0..4], "PWAD") or std.mem.eql(u8, magic[0..4], "IWAD")) {
                try self.mountWad(parent, file);
            } else {
                try self.mountFile(parent, file);
            }
        }

        fn mountDir(self: *Self, parent: *Folder, dir: NamedDir) anyerror!void {
            const thisdir = try self.makeFolder(parent, dir.name);
            var iter = dir.inner.iterateAssumeFirstIteration();

            while (try iter.next()) |dirent| {
                switch (dirent.kind) {
                    .directory => {
                        var child = try std.fs.openDirAbsolute(dirent.name, .{});
                        defer child.close();

                        try self.mountDir(thisdir, NamedDir{
                            .name = dirent.name,
                            .inner = &child,
                        });
                    },
                    .file => {},
                    else => continue,
                }
            }
        }

        fn mountFile(self: *Self, parent: *Folder, file: NamedFile) anyerror!void {
            const bytes = try file.inner.readToEndAlloc(self.alloc.content, self.max_read_size);
            try self.mountMem(parent, file.name, bytes);
        }

        fn mountMem(
            self: *Self,
            parent: *Folder,
            name: VirtualPath,
            bytes: []u8,
        ) std.mem.Allocator.Error!void {
            const n = try self.alloc.metadata.dupe(u8, name);
            errdefer self.alloc.metadata.free(name);

            const entry = try self.alloc.metadata.create(Entry);
            entry.* = Entry{
                .name = n,
                .parent = parent,
                .content = bytes,
                .detail = Entry.Detail{
                    .encryption = .none,
                    .state = .unmodified,
                    .state_locked = false,
                    .locked = false,
                },
            };
            self.total_entries += 1;
            try parent.entries.append(self.alloc.metadata, entry);

            if (config.stem8_map) {
                try self.stem8_map.put(self.alloc.metadata, entry.stem8(), entry);
            }
        }

        fn mountWad(self: *Self, parent: *Folder, file: NamedFile) anyerror!void {
            const thisdir = try self.makeFolder(parent, file.name);
            var reader = try wad.LumpIterator(*std.fs.File).init(file.inner);

            while (try reader.next(self.alloc.content)) |lump| {
                try self.mountMem(thisdir, lump.name(), lump.data);
            }
        }

        fn makeFolder(self: *Self, parent: *Folder, name: []const u8) std.mem.Allocator.Error!*Folder {
            const folder = try self.alloc.metadata.create(Folder);

            folder.* = Folder{
                .name = try self.alloc.metadata.dupe(u8, std.fs.path.basename(name)),
                .parent = parent,
                .entries = .{},
                .subfolders = .{},
            };

            try parent.subfolders.append(self.alloc.metadata, folder);
            self.total_folders += 1;
            return folder;
        }

        fn freeEntry(self: *Self, entry: *const Entry) void {
            self.alloc.content.free(entry.content);
            self.alloc.metadata.free(entry.name);
            self.alloc.metadata.destroy(entry);
        }
    };
}

pub const Folder = struct {
    name: VirtualPath,
    /// Only `null` for the root.
    parent: ?*const Folder,
    entries: std.ArrayListUnmanaged(*Entry),
    subfolders: std.ArrayListUnmanaged(*Folder),
};

pub const Entry = struct {
    pub const Detail = packed struct(u8) {
        encryption: Encryption,
        state: State,
        state_locked: bool,
        locked: bool,
        _pad: u1 = undefined,
    };

    pub const Encryption = enum(u3) {
        none,
        jaguar,
        blood,
        scrle0,
        txb,
    };

    pub const State = enum(u2) {
        unmodified,
        modified,
        new,
        deleted,
    };

    name: VirtualPath,
    parent: *const Folder,
    content: []const u8,
    detail: Detail,

    pub fn stem8(self: *const Entry) Stem8 {
        const stem = std.fs.path.stem(self.name);
        var ret: Stem8 = [1:0]u8{0} ** 8;
        const copy_len = @min(ret.len, stem.len);
        @memcpy(ret[0..copy_len], stem[0..copy_len]);
        return ret;
    }
};

pub const Error = error{
    RelativePath,
} ||
    std.mem.Allocator.Error ||
    std.fs.Dir.OpenError || std.fs.Dir.IteratorError ||
    std.fs.File.OpenError || std.fs.File.ReadError || std.fs.File.SeekError ||
    std.fs.File.MetadataError ||
    wadload.wad.IterError;

const NamedFile = struct {
    /// Never belongs to `VirtualFs.alloc.content`.
    name: RealPath,
    inner: *std.fs.File,
};

const NamedDir = struct {
    /// Never belongs to `VirtualFs.alloc.content`.
    name: RealPath,
    inner: *std.fs.Dir,
};

const nil_folder = Folder{
    .name = "",
    .parent = null,
    .entries = .{},
    .subfolders = .{},
};

pub const Stem8 = [8:0]u8;

pub fn Stem8Map(V: type) type {
    return std.AutoHashMap(Stem8, V);
}

pub fn Stem8MapUnmanaged(V: type) type {
    return std.AutoHashMapUnmanaged(Stem8, V);
}

test "mount, smoke" {
    const Vfs = VirtualFs(.{ .stem8_map = true });

    const mount_path = try std.fs.cwd().realpathAlloc(std.testing.allocator, "sample/udmf.wad");
    defer std.testing.allocator.free(mount_path);

    var vfs = Vfs.init(std.testing.allocator, std.testing.allocator, 1024 * 1024);
    defer vfs.deinit();
    try vfs.mount(mount_path);

    try std.testing.expectEqual(null, vfs.root.parent);
    try std.testing.expectEqual(2, vfs.total_folders);
    try std.testing.expectEqual(4, vfs.total_entries);
    try std.testing.expectEqual(&vfs.root, vfs.root.subfolders.items[0].parent);

    vfs.clear();
    try std.testing.expectEqual(1, vfs.total_folders);
    try std.testing.expectEqual(0, vfs.total_entries);
}
