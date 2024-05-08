#!/usr/bin/env bash
# Generates a .clangd file for `/nimcache` so that clangd can be used when
# inspecting generated C/C++ files.

NIM_DIR=`choosenim show path`

echo "---" > ./nimcache/.clangd
echo "CompileFlags:" >> ./nimcache/.clangd
echo "  Add: [" >> ./nimcache/.clangd
echo "    '-isystem$NIM_DIR/lib'," >> ./nimcache/.clangd
echo "  ]" >> ./nimcache/.clangd
echo "..." >> ./nimcache/.clangd
