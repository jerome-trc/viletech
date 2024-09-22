const consts = @import("consts.zig");
const funcs = @import("funcs.zig");
const structs = @import("structs.zig");
const vars = @import("vars.zig");

pub export fn I_Init() void {
    funcs.dsda_ResetTimeFunctions(vars.fastdemo);
    funcs.I_InitSound();
}
