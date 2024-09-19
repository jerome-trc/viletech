/// @file
/// @brief Routines for building a Doom map's BLOCKMAP lump.

#pragma once

#include "doomdata.hpp"
#include "tarray.hpp"

class FBlockmapBuilder {
public:
	FBlockmapBuilder(FLevel& level);
	uint16_t* GetBlockmap(int32_t& size);

private:
	FLevel& Level;
	TArray<uint16_t> BlockMap;

	void BuildBlockmap();
	void CreateUnpackedBlockmap(TArray<uint16_t>* blocks, int bmapwidth, int bmapheight);
	void CreatePackedBlockmap(TArray<uint16_t>* blocks, int bmapwidth, int bmapheight);
};
