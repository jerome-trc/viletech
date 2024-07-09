pub const c = @cImport({
    @cInclude("zdfs/zdfs.h");
});

pub const ZdfsError = error{
    FileSysInitNull,
};
