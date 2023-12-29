#ifndef __PROCESSOR_H__
#define __PROCESSOR_H__

#include "sc_man.hpp"
#ifdef _MSC_VER
#pragma once
#endif

#include <zlib.h>

#include "wad.hpp"
#include "doomdata.hpp"
#include "tarray.hpp"
#include "nodebuild.hpp"
#include "zdbsp.h"

class ZLibOut {
public:
	ZLibOut(FWadWriter& out);
	~ZLibOut();

	ZLibOut& operator<<(BYTE);
	ZLibOut& operator<<(WORD);
	ZLibOut& operator<<(SWORD);
	ZLibOut& operator<<(DWORD);
	ZLibOut& operator<<(fixed_t);
	void Write(BYTE* data, int len);

private:
	enum {
		BUFFER_SIZE = 8192
	};

	z_stream Stream;
	BYTE Buffer[BUFFER_SIZE];

	FWadWriter& Out;
};

class StringBuffer {
	const static size_t BLOCK_SIZE = 100000;
	const static size_t BLOCK_ALIGN = sizeof(size_t);

	TDeletingArray<char*> blocks;
	size_t currentindex;

	char* Alloc(size_t size) {
		if (currentindex + size >= BLOCK_SIZE) {
			// Block is full - get a new one!
			char* newblock = new char[BLOCK_SIZE];
			blocks.Push(newblock);
			currentindex = 0;
		}
		size = (size + BLOCK_ALIGN - 1) & ~(BLOCK_ALIGN - 1);
		char* p = blocks[blocks.Size() - 1] + currentindex;
		currentindex += size;
		return p;
	}

public:
	StringBuffer() {
		currentindex = BLOCK_SIZE;
	}

	char* Copy(const char* p) {
		return p != NULL ? strcpy(Alloc(strlen(p) + 1), p) : NULL;
	}
};

class FProcessor {
public:
	FProcessor(FWadReader& inwad, int lump);
	void Process(const zdbsp_NodeConfig* config);
	void Write(FWadWriter& out);
	zdbsp_NodeVersion NodeVersion() const;

	bool build_nodes = true, build_gl_nodes = false;
	bool conform_nodes = false, gl_only = false;
	bool check_poly_objs = true, no_prune = false;
	bool write_comments = false, v5gl = false;
	bool compress_nodes = false, compress_gl_nodes = false, force_compression = false;

	zdbsp_RejectMode reject_mode = ZDBSP_ERM_DONTTOUCH;
	zdbsp_BlockmapMode blockmap_mode = ZDBSP_EBM_REBUILD;

	const FLevel& get_level() const {
		return this->Level;
	}

private:
	void LoadUDMF();
	void LoadThings();
	void LoadLines();
	void LoadVertices();
	void LoadSides();
	void LoadSectors();
	void GetPolySpots();

	zdbsp_NodeEx* NodesToEx(const zdbsp_NodeRaw* nodes, int count);
	zdbsp_SubsectorEx* SubsectorsToEx(const zdbsp_SubsectorRaw* ssec, int count);
	zdbsp_SegGlEx* SegGLsToEx(const zdbsp_SegGl* segs, int count);

	BYTE* FixReject(const BYTE* oldreject);
	bool CheckForFracSplitters(const zdbsp_NodeEx* nodes, int count) const;

	void WriteLines(FWadWriter& out);
	void WriteVertices(FWadWriter& out, int count);
	void WriteSectors(FWadWriter& out);
	void WriteSides(FWadWriter& out);
	void WriteSegs(FWadWriter& out);
	void WriteSSectors(FWadWriter& out) const;
	void WriteNodes(FWadWriter& out) const;
	void WriteBlockmap(FWadWriter& out);
	void WriteReject(FWadWriter& out);

	void WriteGLVertices(FWadWriter& out, bool v5);
	void WriteGLSegs(FWadWriter& out, bool v5);
	void WriteGLSegs5(FWadWriter& out);
	void WriteGLSSect(FWadWriter& out, bool v5);
	void WriteGLNodes(FWadWriter& out, bool v5);

	void WriteBSPZ(FWadWriter& out, const char* label);
	void WriteGLBSPZ(FWadWriter& out, const char* label);

	void WriteVerticesZ(ZLibOut& out, const zdbsp_VertexEx* verts, int orgverts, int newverts);
	void WriteSubsectorsZ(ZLibOut& out, const zdbsp_SubsectorEx* subs, int numsubs);
	void WriteSegsZ(ZLibOut& out, const zdbsp_SegEx* segs, int numsegs);
	void WriteGLSegsZ(ZLibOut& out, const zdbsp_SegGlEx* segs, int numsegs, int nodever);
	void WriteNodesZ(ZLibOut& out, const zdbsp_NodeEx* nodes, int numnodes, int nodever);

	void WriteBSPX(FWadWriter& out, const char* label);
	void WriteGLBSPX(FWadWriter& out, const char* label);

	void WriteVerticesX(FWadWriter& out, const zdbsp_VertexEx* verts, int orgverts, int newverts);
	void WriteSubsectorsX(FWadWriter& out, const zdbsp_SubsectorEx* subs, int numsubs);
	void WriteSegsX(FWadWriter& out, const zdbsp_SegEx* segs, int numsegs);
	void WriteGLSegsX(FWadWriter& out, const zdbsp_SegGlEx* segs, int numsegs, int nodever);
	void WriteNodesX(FWadWriter& out, const zdbsp_NodeEx* nodes, int numnodes, int nodever);

	void WriteNodes2(FWadWriter& out, const char* name, const zdbsp_NodeEx* zaNodes, int count)
		const;
	void WriteSSectors2(FWadWriter& out, const char* name, const zdbsp_SubsectorEx* zaSubs, int count)
		const;
	void WriteNodes5(FWadWriter& out, const char* name, const zdbsp_NodeEx* zaNodes, int count)
		const;
	void WriteSSectors5(FWadWriter& out, const char* name, const zdbsp_SubsectorEx* zaSubs, int count)
		const;

	const char* ParseKey(const char*& value);
	bool CheckKey(const char*& key, const char*& value);
	void ParseThing(IntThing* th);
	void ParseLinedef(IntLineDef* ld);
	void ParseSidedef(IntSideDef* sd);
	void ParseSector(IntSector* sec);
	void ParseVertex(zdbsp_VertexEx* vt, IntVertex* vtp);
	void ParseMapProperties();
	void ParseTextMap(int lump);

	int CheckInt(const char* key);
	double CheckFloat(const char* key);
	fixed_t CheckFixed(const char* key);

	void WriteProps(FWadWriter& out, TArray<zdbsp_UdmfKey>& props);
	void WriteIntProp(FWadWriter& out, const char* key, int value);
	void WriteThingUDMF(FWadWriter& out, IntThing* th, int num);
	void WriteLinedefUDMF(FWadWriter& out, IntLineDef* ld, int num);
	void WriteSidedefUDMF(FWadWriter& out, IntSideDef* sd, int num);
	void WriteSectorUDMF(FWadWriter& out, IntSector* sec, int num);
	void WriteVertexUDMF(FWadWriter& out, IntVertex* vt, int num);
	void WriteTextMap(FWadWriter& out);
	void WriteUDMF(FWadWriter& out);

	FLevel Level;

	TArray<FNodeBuilder::FPolyStart> PolyStarts;
	TArray<FNodeBuilder::FPolyStart> PolyAnchors;

	bool Extended;
	bool isUDMF;
	zdbsp_NodeVersion node_version = ZDBSP_NODEVERS_UNKNOWN;

	FWadReader& Wad;
	Scanner scanner;
	StringBuffer stbuf;
	int Lump;
};

#endif //__PROCESSOR_H__
