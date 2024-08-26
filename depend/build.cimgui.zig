const builtin = @import("builtin");
const std = @import("std");

pub fn link(b: *std.Build, compile: *std.Build.Step.Compile) void {
    var cxx_flags: []const []const u8 = if (builtin.os.tag == .windows)
        &[_][]const u8{ "--std=c++20", "-DIMGUI_IMPL_API=extern \"C\" __declspec(dllexport)" }
    else
        &[_][]const u8{ "--std=c++20", "-DIMGUI_IMPL_API=extern \"C\"" };

    if (false) {
        const flags = b.run(&[_][]const u8{ "pkg-config", "--cflags-only-I", "sdl2" });
        var iter = std.mem.splitScalar(u8, flags, ' ');

        while (iter.next()) |flag| {
            const f = std.mem.trim(u8, flag, " \n\r\t");
            const dir = std.mem.trim(u8, f, "-I");
            compile.addSystemIncludePath(.{ .cwd_relative = dir });

            cxx_flags = std.mem.concat(
                b.allocator,
                []const u8,
                &[2][]const []const u8{ cxx_flags, &[1][]const u8{f} },
            ) catch unreachable;
        }
    }

    compile.addCSourceFiles(.{
        .root = b.path("depend/imgui"),
        .flags = cxx_flags,
        .files = &[_][]const u8{
            "imgui_impl_sdl2.cpp",
            "imgui_impl_opengl3.cpp",
            "cimgui.cpp",
            "imgui_demo.cpp",
            "imgui_draw.cpp",
            "imgui_tables.cpp",
            "imgui_widgets.cpp",
            "imgui.cpp",
        },
    });

    compile.addSystemIncludePath(b.path("depend/imgui"));
}

pub fn moduleLink(b: *std.Build, module: *std.Build.Module) void {
    if (builtin.os.tag != .windows) {
        const flags = b.run(&[_][]const u8{ "pkg-config", "--cflags-only-I", "sdl2" });
        var iter = std.mem.splitScalar(u8, flags, ' ');

        while (iter.next()) |flag| {
            const f = std.mem.trim(u8, flag, " \n\r\t");
            const dir = std.mem.trim(u8, f, "-I");
            module.addSystemIncludePath(.{ .cwd_relative = dir });
        }
    }

    module.addSystemIncludePath(b.path("depend/imgui"));
}
