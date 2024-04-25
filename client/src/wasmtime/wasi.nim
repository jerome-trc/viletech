const hWasi = "<wasi.h>"

type
    WasiConfigObj* {.header: hWasi, importc: "wasi_config_t".} = object
    WasiConfig* = ptr WasiConfigObj

proc init*(_: typedesc[WasiConfig]): WasiConfig
    {.header: hWasi, importc: "wasi_config_new".}

proc delete*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_delete".}

proc inheritArgv*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_inherit_argv".}

proc inheritEnv*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_inherit_env".}

proc inheritStdin*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_inherit_stdin".}

proc inheritStdout*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_inherit_stdout".}

proc inheritStderr*(this: WasiConfig)
    {.header: hWasi, importc: "wasi_config_inherit_stderr".}
