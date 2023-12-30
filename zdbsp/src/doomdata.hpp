#ifndef __DOOMDATA_H__
#define __DOOMDATA_H__

#ifdef _MSC_VER
#pragma once
#endif

#include "tarray.hpp"
#include "common.hpp"

enum {
	BOXTOP,
	BOXBOTTOM,
	BOXLEFT,
	BOXRIGHT
};

struct MapSideDef {
	short textureoffset;
	short rowoffset;
	char toptexture[8];
	char bottomtexture[8];
	char midtexture[8];
	uint16_t sector;
};

struct IntSideDef {
	// the first 5 values are only used for binary format maps
	short textureoffset;
	short rowoffset;
	char toptexture[8];
	char bottomtexture[8];
	char midtexture[8];

	int sector;

	TArray<zdbsp_UdmfKey> props;
};

struct MapLineDef {
	uint16_t v1;
	uint16_t v2;
	short flags;
	short special;
	short tag;
	uint16_t sidenum[2];
};

struct MapLineDef2 {
	uint16_t v1;
	uint16_t v2;
	short flags;
	unsigned char special;
	unsigned char args[5];
	uint16_t sidenum[2];
};

struct IntLineDef {
	uint32_t v1;
	uint32_t v2;
	int flags;
	int special;
	int args[5];
	uint32_t sidenum[2];

	TArray<zdbsp_UdmfKey> props;
};

struct MapSector {
	short floorheight;
	short ceilingheight;
	char floorpic[8];
	char ceilingpic[8];
	short lightlevel;
	short special;
	short tag;
};

struct IntSector {
	// none of the sector properties are used by the node builder
	// so there's no need to store them in their expanded form for
	// UDMF. Just storing the UDMF keys and leaving the binary fields
	// empty is enough
	MapSector data;

	TArray<zdbsp_UdmfKey> props;
};

#define NF_SUBSECTOR 0x8000
#define NFX_SUBSECTOR 0x80000000

struct IntThing {
	unsigned short thingid;
	zdbsp_I16F16 x; // full precision coordinates for UDMF support
	zdbsp_I16F16 y;
	// everything else is not needed or has no extended form in UDMF
	short z;
	short angle;
	short type;
	short flags;
	char special;
	char args[5];

	TArray<zdbsp_UdmfKey> props;
};

struct IntVertex {
	TArray<zdbsp_UdmfKey> props;
};

struct FLevel {
	FLevel();
	~FLevel();

	zdbsp_VertexEx* Vertices;
	int32_t NumVertices;
	TArray<IntVertex> VertexProps;
	TArray<IntSideDef> Sides;
	TArray<IntLineDef> Lines;
	TArray<IntSector> Sectors;
	TArray<IntThing> Things;
	zdbsp_SubsectorEx* Subsectors;
	int32_t NumSubsectors;
	zdbsp_SegEx* Segs;
	int32_t NumSegs;
	zdbsp_NodeEx* Nodes;
	int32_t NumNodes;
	uint16_t* Blockmap;
	int32_t BlockmapSize;
	uint8_t* Reject;
	int32_t RejectSize;

	zdbsp_SubsectorEx* GLSubsectors;
	int32_t NumGLSubsectors;
	zdbsp_SegGlEx* GLSegs;
	int32_t NumGLSegs;
	zdbsp_NodeEx* GLNodes;
	int32_t NumGLNodes;
	zdbsp_VertexEx* GLVertices;
	int32_t NumGLVertices;
	uint8_t* GLPVS;
	int32_t GLPVSSize;

	int NumOrgVerts;

	uint32_t* OrgSectorMap;
	int NumOrgSectors;

	zdbsp_I16F16 MinX, MinY, MaxX, MaxY;

	TArray<zdbsp_UdmfKey> props;

	void find_map_bounds();
	void remove_extra_lines();
	void RemoveExtraSides();
	void remove_extra_sectors();

	uint32_t NumSides() const {
		return Sides.Size();
	}
	uint32_t NumLines() const {
		return Lines.Size();
	}
	uint32_t NumSectors() const {
		return Sectors.Size();
	}
	uint32_t NumThings() const {
		return Things.Size();
	}
};

const int BLOCKSIZE = 128;
const int BLOCKFRACSIZE = BLOCKSIZE << FRACBITS;
const int BLOCKBITS = 7;
const int BLOCKFRACBITS = FRACBITS + 7;

#endif //__DOOMDATA_H__
