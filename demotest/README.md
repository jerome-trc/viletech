# Demo Accuracy Regression Test Suite

To run:
1. Populate `/sample/iwads` with `DOOM.WAD`, `DOOM2.WAD`, `HERETIC.WAD`, and `HEXEN.WAD`.
2. Populate `/sample/pwads` with `rush.wad` and `Valiant.wad`.
4. Run the following:
```bash
mkdir -p build
zig build install -p -Doptimize=ReleaseSafe
cmake --build ./build --config Release --target all --
zig build demotest
```
