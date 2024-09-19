/// @file
/// @brief WAD-handling routines.

/*

Copyright (C) 2002-2006 Randy Heit

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

*/

#include <cstring>

#include "wad.hpp"
#include "common.hpp"

static const char MapLumpNames[12][9] = { "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES",
										  "SEGS",	"SSECTORS", "NODES",	"SECTORS",
										  "REJECT", "BLOCKMAP", "BEHAVIOR", "SCRIPTS" };

static const bool MapLumpRequired[12] = {
	true, // THINGS
	true, // LINEDEFS
	true, // SIDEDEFS
	true, // VERTEXES
	false, // SEGS
	false, // SSECTORS
	false, // NODES
	true, // SECTORS
	false, // REJECT
	false, // BLOCKMAP
	false, // BEHAVIOR
	false // SCRIPTS
};

static const char GLLumpNames[5][9] = { "GL_VERT", "GL_SEGS", "GL_SSECT", "GL_NODES", "GL_PVS" };

FWadReader::FWadReader(const uint8_t* const bytes) : bytes(bytes), Lumps(nullptr), cursor(0) {
	if (bytes == nullptr) {
		throw std::runtime_error("Received a null pointer illegally");
	}

	auto header = reinterpret_cast<const WadHeader*>(bytes);

	if (header->Magic[0] != 'P' && header->Magic[0] != 'I' && header->Magic[1] != 'W' &&
		header->Magic[2] != 'A' && header->Magic[3] != 'D') {
		throw std::runtime_error("Input buffer is not a WAD");
	}

	auto lmp_count = (size_t)little_long(header->NumLumps);
	auto dir_offs = (size_t)little_long(header->Directory);
	auto directory = reinterpret_cast<const WadLump*>(bytes + dir_offs);

	this->Lumps = new WadLump[lmp_count];
	this->Header.NumLumps = lmp_count;
	this->Header.Directory = dir_offs;
	std::memcpy(this->Lumps, directory, lmp_count * sizeof(WadLump));

	for (size_t i = 0; i < lmp_count; ++i) {
		auto lmp = &this->Lumps[i];
		lmp->FilePos = little_long(lmp->FilePos);
		lmp->Size = little_long(lmp->Size);
	}
}

FWadReader::~FWadReader() {
	if (Lumps)
		delete[] Lumps;
}

bool FWadReader::IsIWAD() const {
	return Header.Magic[0] == 'I';
}

int FWadReader::NumLumps() const {
	return Header.NumLumps;
}

int FWadReader::FindLump(const char* name, int index) const {
	if (index < 0) {
		index = 0;
	}
	for (; index < Header.NumLumps; ++index) {
		if (strnicmp(Lumps[index].Name, name, 8) == 0) {
			return index;
		}
	}
	return -1;
}

int FWadReader::FindMapLump(const char* name, int map) const {
	int i, j, k;
	++map;

	for (i = 0; i < 12; ++i) {
		if (strnicmp(MapLumpNames[i], name, 8) == 0) {
			break;
		}
	}
	if (i == 12) {
		return -1;
	}

	for (j = k = 0; j < 12; ++j) {
		if (strnicmp(Lumps[map + k].Name, MapLumpNames[j], 8) == 0) {
			if (i == j) {
				return map + k;
			}
			k++;
		}
	}
	return -1;
}

bool FWadReader::isUDMF(int index) const {
	index++;

	if (strnicmp(Lumps[index].Name, "TEXTMAP", 8) == 0) {
		// UDMF map
		return true;
	}
	return false;
}

bool FWadReader::IsMap(int index) const {
	int i, j;

	if (isUDMF(index))
		return true;

	index++;

	for (i = j = 0; i < 12; ++i) {
		if (strnicmp(Lumps[index + j].Name, MapLumpNames[i], 8) != 0) {
			if (MapLumpRequired[i]) {
				return false;
			}
		} else {
			j++;
		}
	}
	return true;
}

int FWadReader::FindGLLump(const char* name, int glheader) const {
	int i, j, k;
	++glheader;

	for (i = 0; i < 5; ++i) {
		if (strnicmp(Lumps[glheader + i].Name, name, 8) == 0) {
			break;
		}
	}
	if (i == 5) {
		return -1;
	}

	for (j = k = 0; j < 5; ++j) {
		if (strnicmp(Lumps[glheader + k].Name, GLLumpNames[j], 8) == 0) {
			if (i == j) {
				return glheader + k;
			}
			k++;
		}
	}
	return -1;
}

bool FWadReader::IsGLNodes(int index) const {
	if (index + 4 >= Header.NumLumps) {
		return false;
	}
	if (Lumps[index].Name[0] != 'G' || Lumps[index].Name[1] != 'L' || Lumps[index].Name[2] != '_') {
		return false;
	}
	index++;
	for (int i = 0; i < 4; ++i) {
		if (strnicmp(Lumps[i + index].Name, GLLumpNames[i], 8) != 0) {
			return false;
		}
	}
	return true;
}

int FWadReader::SkipGLNodes(int index) const {
	index++;
	for (int i = 0; i < 5 && index < Header.NumLumps; ++i, ++index) {
		if (strnicmp(Lumps[index].Name, GLLumpNames[i], 8) != 0) {
			break;
		}
	}
	return index;
}

bool FWadReader::MapHasBehavior(int map) const {
	return FindMapLump("BEHAVIOR", map) != -1;
}

int FWadReader::NextMap(int index) const {
	if (index < 0) {
		index = 0;
	} else {
		index++;
	}
	for (; index < Header.NumLumps; ++index) {
		if (IsMap(index)) {
			return index;
		}
	}
	return -1;
}

int FWadReader::LumpAfterMap(int i) const {
	int j, k;

	if (isUDMF(i)) {
		// UDMF map
		i += 2;
		while (strnicmp(Lumps[i].Name, "ENDMAP", 8) != 0 && i < Header.NumLumps) {
			i++;
		}
		return i + 1; // one lump after ENDMAP
	}

	i++;
	for (j = k = 0; j < 12; ++j) {
		if (strnicmp(Lumps[i + k].Name, MapLumpNames[j], 8) != 0) {
			if (MapLumpRequired[j]) {
				break;
			}
		} else {
			k++;
		}
	}
	return i + k;
}

void FWadReader::SafeRead(void* buffer, size_t size) {
	std::memcpy(buffer, this->bytes + this->cursor, size);
	this->cursor += size;
}

const char* FWadReader::LumpName(int lump) {
	static char name[9];
	strncpy(name, Lumps[lump].Name, 8);
	name[8] = 0;
	return name;
}

FWadWriter::FWadWriter(uint8_t* dest, bool iwad) : dest(dest), cursor(0) {
	WadHeader head;

	if (iwad) {
		head.Magic[0] = 'I';
	} else {
		head.Magic[0] = 'P';
	}

	head.Magic[1] = 'W';
	head.Magic[2] = 'A';
	head.Magic[3] = 'D';

	this->SafeWrite(dest, sizeof(WadHeader));
}

FWadWriter::~FWadWriter() { }

void FWadWriter::Close() { }

void FWadWriter::CreateLabel(const char* name) {
	WadLump lump;

	strncpy(lump.Name, name, 8);
	lump.FilePos = little_long(this->cursor);
	lump.Size = 0;
	Lumps.Push(lump);
}

void FWadWriter::WriteLump(const char* name, const void* data, int len) {
	WadLump lump;

	strncpy(lump.Name, name, 8);
	lump.FilePos = little_long(this->cursor);
	lump.Size = little_long(len);
	Lumps.Push(lump);

	SafeWrite(data, len);
}

void FWadWriter::CopyLump(FWadReader& wad, int lump) {
	uint8_t* data;
	int32_t size;

	ReadLump<uint8_t>(wad, lump, data, size);
	if (data != NULL) {
		WriteLump(wad.LumpName(lump), data, size);
		delete[] data;
	}
}

void FWadWriter::StartWritingLump(const char* name) {
	CreateLabel(name);
}

void FWadWriter::AddToLump(const void* data, int len) {
	SafeWrite(data, len);
	Lumps[Lumps.Size() - 1].Size += len;
}

void FWadWriter::SafeWrite(const void* buffer, size_t size) {
	std::memcpy(this->dest, buffer, size);
	this->cursor += size;
}

FWadWriter& FWadWriter::operator<<(uint8_t val) {
	AddToLump(&val, 1);
	return *this;
}

FWadWriter& FWadWriter::operator<<(uint16_t val) {
	val = little_short(val);
	AddToLump((uint8_t*)&val, 2);
	return *this;
}

FWadWriter& FWadWriter::operator<<(int16_t val) {
	val = little_short(val);
	AddToLump((uint8_t*)&val, 2);
	return *this;
}

FWadWriter& FWadWriter::operator<<(uint32_t val) {
	val = little_long(val);
	AddToLump((uint8_t*)&val, 4);
	return *this;
}

FWadWriter& FWadWriter::operator<<(znbx_I16F16 val) {
	val = little_long(val);
	AddToLump((uint8_t*)&val, 4);
	return *this;
}
