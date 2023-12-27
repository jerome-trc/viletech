#ifndef __DOOMDATA_H__
#define __DOOMDATA_H__

#ifdef _MSC_VER
#pragma once
#endif

#include "tarray.hpp"
#include "common.hpp"

#include "zdbsp.h"

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
	WORD sector;
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
	WORD v1;
	WORD v2;
	short flags;
	short special;
	short tag;
	WORD sidenum[2];
};

struct MapLineDef2 {
	WORD v1;
	WORD v2;
	short flags;
	unsigned char special;
	unsigned char args[5];
	WORD sidenum[2];
};

struct IntLineDef {
	DWORD v1;
	DWORD v2;
	int flags;
	int special;
	int args[5];
	DWORD sidenum[2];

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
	fixed_t x; // full precision coordinates for UDMF support
	fixed_t y;
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

	zdbsp_VertexWide* Vertices;
	size_t NumVertices;
	TArray<IntVertex> VertexProps;
	TArray<IntSideDef> Sides;
	TArray<IntLineDef> Lines;
	TArray<IntSector> Sectors;
	TArray<IntThing> Things;
	zdbsp_SubsectorEx* Subsectors;
	size_t NumSubsectors;
	zdbsp_SegEx* Segs;
	size_t NumSegs;
	zdbsp_NodeEx* Nodes;
	size_t NumNodes;
	WORD* Blockmap;
	size_t BlockmapSize;
	BYTE* Reject;
	size_t RejectSize;

	zdbsp_SubsectorEx* GLSubsectors;
	size_t NumGLSubsectors;
	zdbsp_SegGlEx* GLSegs;
	size_t NumGLSegs;
	zdbsp_NodeEx* GLNodes;
	size_t NumGLNodes;
	zdbsp_VertexWide* GLVertices;
	size_t NumGLVertices;
	BYTE* GLPVS;
	size_t GLPVSSize;

	int NumOrgVerts;

	DWORD* OrgSectorMap;
	int NumOrgSectors;

	fixed_t MinX, MinY, MaxX, MaxY;

	TArray<zdbsp_UdmfKey> props;

	void FindMapBounds();
	void RemoveExtraLines();
	void RemoveExtraSides();
	void RemoveExtraSectors();

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
