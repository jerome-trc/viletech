const builtin = @import("builtin");
const std = @import("std");

pub fn build(b: *std.Build, compile: *std.Build.Step.Compile) void {
    const cxx_flags = if (builtin.os.tag == .windows)
        [_][]const u8{ "--std=c++20", "-DIMGUI_IMPL_API=extern \"C\" __declspec(dllexport)" }
    else
        [_][]const u8{ "--std=c++20", "-DIMGUI_IMPL_API=extern \"C\"" };

    compile.addCSourceFiles(.{
        .root = b.path("depend/imgui"),
        .flags = &cxx_flags,
        .files = &[_][]const u8{
            "cimgui.cpp",
            "imgui_demo.cpp",
            "imgui_draw.cpp",
            "imgui_tables.cpp",
            "imgui_widgets.cpp",
            "imgui.cpp",
        },
    });

    compile.addIncludePath(b.path("depend/imgui"));
}
