# ZMSX

A fork of ZMusic, GZDoom's MIDI music system as a standalone library.

Compile instructions are pretty simple for most systems.

```
git clone https://github.com/jerome-trc/zmsx.git
mkdir zmsx/build
cd zmsx/build
cmake -DCMAKE_BUILD_TYPE=Release ..
cmake --build .
```

On Unix/Linux you may also supply `sudo make install` in the build folder to push the compiled library directly into the file system so that it can be found by the previously mentioned projects.
