#ifndef __WAD_H__
#define __WAD_H__

#include <memory>
#ifdef _MSC_VER
#pragma once
#endif

#include <stdio.h>
#include <string.h>

#include "common.hpp"
#include "tarray.hpp"

struct WadHeader {
	char Magic[4];
	int32_t NumLumps;
	int32_t Directory;
};

struct WadLump {
	int32_t FilePos;
	int32_t Size;
	char Name[8];
};

class FWadReader {
public:
	FWadReader(const uint8_t* bytes);
	~FWadReader();

	bool IsIWAD() const;
	bool isUDMF(int lump) const;
	int FindLump(const char* name, int index = 0) const;
	int FindMapLump(const char* name, int map) const;
	int FindGLLump(const char* name, int glheader) const;
	const char* LumpName(int lump);
	bool IsMap(int index) const;
	bool IsGLNodes(int index) const;
	int SkipGLNodes(int index) const;
	bool MapHasBehavior(int map) const;
	int NextMap(int startindex) const;
	int LumpAfterMap(int map) const;
	int NumLumps() const;

	void SafeRead(void* buffer, size_t size);

	template<class T>
	friend void ReadLump(znbx_SliceU8 slice, T*& data, int32_t& size);

	// VC++ 6 does not support template member functions in non-template classes!
	template<class T>
	friend void ReadLump(FWadReader& wad, int index, T*& data, int32_t& size);

private:
	const uint8_t* bytes;
	WadHeader Header;
	WadLump* Lumps;
	size_t cursor;
};

template<class T>
void read_lump(znbx_SliceU8 slice, T*& data, int32_t& size) {
	size = slice.len / sizeof(T);
	data = new T[size];
	memcpy(data, slice.ptr, size * sizeof(T));
}

template<class T>
void ReadLump(FWadReader& wad, int index, T*& data, int32_t& size) {
	if ((unsigned)index >= (unsigned)wad.Header.NumLumps) {
		data = NULL;
		size = 0;
		return;
	}

	wad.cursor = (size_t)wad.Lumps[index].FilePos;
	size = wad.Lumps[index].Size / sizeof(T);
	data = new T[size];
	wad.SafeRead(data, size * sizeof(T));
}

template<class T>
void ReadMapLump(FWadReader& wad, const char* name, int index, T*& data, int32_t& size) {
	read_lump(wad, wad.FindMapLump(name, index), data, size);
}

class FWadWriter {
public:
	FWadWriter(uint8_t* dest, bool iwad);
	~FWadWriter();

	void CreateLabel(const char* name);
	void WriteLump(const char* name, const void* data, int len);
	void CopyLump(FWadReader& wad, int lump);
	void Close();

	// Routines to write a lump in segments.
	void StartWritingLump(const char* name);
	void AddToLump(const void* data, int len);

	FWadWriter& operator<<(uint8_t);
	FWadWriter& operator<<(uint16_t);
	FWadWriter& operator<<(int16_t);
	FWadWriter& operator<<(uint32_t);
	FWadWriter& operator<<(znbx_I16F16);

private:
	TArray<WadLump> Lumps;
	uint8_t* dest;
	size_t cursor;

	void SafeWrite(const void* buffer, size_t size);
};

#endif //__WAD_H__
