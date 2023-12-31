#ifndef __PROCESSOR_H__
#define __PROCESSOR_H__


#ifdef _MSC_VER
#pragma once
#endif

#include <zlib.h>

#include "sc_man.hpp"
#include "wad.hpp"
#include "doomdata.hpp"
#include "tarray.hpp"
#include "nodebuild.hpp"
#include "znbx.h"

class ZLibOut {
public:
	ZLibOut(FWadWriter& out);
	~ZLibOut();

	ZLibOut& operator<<(uint8_t);
	ZLibOut& operator<<(uint16_t);
	ZLibOut& operator<<(int16_t);
	ZLibOut& operator<<(uint32_t);
	ZLibOut& operator<<(znbx_I16F16);
	void Write(uint8_t* data, int len);

private:
	enum {
		BUFFER_SIZE = 8192
	};

	z_stream Stream;
	uint8_t Buffer[BUFFER_SIZE];

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
	DELETE_COPIERS(FProcessor)

public:
	FProcessor(znbx_Level, bool extended);
	FProcessor(znbx_LevelUdmf);

	void Process(const znbx_NodeConfig* config);
	znbx_NodeVersion get_node_version() const;

	bool build_nodes = true, build_gl_nodes = false;
	bool conform_nodes = false, gl_only = false;
	bool check_poly_objs = true, no_prune = false;
	bool write_comments = false, v5gl = false;
	bool compress_nodes = false, compress_gl_nodes = false, force_compression = false;

	znbx_RejectMode reject_mode = ZNBX_ERM_DONTTOUCH;
	znbx_BlockmapMode blockmap_mode = ZNBX_EBM_REBUILD;

	const FLevel& get_level() const {
		return this->Level;
	}

private:
	explicit FProcessor();

	void load_lines(znbx_SliceU8);
	void load_lines_ext(znbx_SliceU8);
	void load_sectors(znbx_SliceU8);
	void load_sides(znbx_SliceU8);
	void load_things(znbx_SliceU8);
	void load_things_ext(znbx_SliceU8);
	void load_vertices(znbx_SliceU8);

	void finish_load();

	void get_poly_spots();

	znbx_NodeEx* NodesToEx(const znbx_NodeRaw* nodes, int count);
	znbx_SubsectorEx* SubsectorsToEx(const znbx_SubsectorRaw* ssec, int count);
	znbx_SegGlEx* SegGLsToEx(const znbx_SegGl* segs, int count);

	uint8_t* fix_reject(const uint8_t* oldreject);
	bool CheckForFracSplitters(const znbx_NodeEx* nodes, int count) const;

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

	void WriteVerticesZ(ZLibOut& out, const znbx_VertexEx* verts, int orgverts, int newverts);
	void WriteSubsectorsZ(ZLibOut& out, const znbx_SubsectorEx* subs, int numsubs);
	void WriteSegsZ(ZLibOut& out, const znbx_SegEx* segs, int numsegs);
	void WriteGLSegsZ(ZLibOut& out, const znbx_SegGlEx* segs, int numsegs, int nodever);
	void WriteNodesZ(ZLibOut& out, const znbx_NodeEx* nodes, int numnodes, int nodever);

	void WriteBSPX(FWadWriter& out, const char* label);
	void WriteGLBSPX(FWadWriter& out, const char* label);

	void WriteVerticesX(FWadWriter& out, const znbx_VertexEx* verts, int orgverts, int newverts);
	void WriteSubsectorsX(FWadWriter& out, const znbx_SubsectorEx* subs, int numsubs);
	void WriteSegsX(FWadWriter& out, const znbx_SegEx* segs, int numsegs);
	void WriteGLSegsX(FWadWriter& out, const znbx_SegGlEx* segs, int numsegs, int nodever);
	void WriteNodesX(FWadWriter& out, const znbx_NodeEx* nodes, int numnodes, int nodever);

	void WriteNodes2(FWadWriter& out, const char* name, const znbx_NodeEx* zaNodes, int count)
		const;
	void WriteSSectors2(FWadWriter& out, const char* name, const znbx_SubsectorEx* zaSubs, int count)
		const;
	void WriteNodes5(FWadWriter& out, const char* name, const znbx_NodeEx* zaNodes, int count)
		const;
	void WriteSSectors5(FWadWriter& out, const char* name, const znbx_SubsectorEx* zaSubs, int count)
		const;

	const char* ParseKey(const char*& value);
	bool CheckKey(const char*& key, const char*& value);
	void ParseThing(IntThing* th);
	void ParseLinedef(IntLineDef* ld);
	void ParseSidedef(IntSideDef* sd);
	void ParseSector(IntSector* sec);
	void ParseVertex(znbx_VertexEx* vt, IntVertex* vtp);
	void ParseMapProperties();
	void ParseTextMap(znbx_SliceU8);

	int CheckInt(const char* key);
	double CheckFloat(const char* key);
	znbx_I16F16 CheckFixed(const char* key);

	void WriteProps(FWadWriter& out, TArray<znbx_UdmfKey>& props);
	void WriteIntProp(FWadWriter& out, const char* key, int value);
	void WriteThingUDMF(FWadWriter& out, IntThing* th, int num);
	void WriteLinedefUDMF(FWadWriter& out, IntLineDef* ld, int num);
	void WriteSidedefUDMF(FWadWriter& out, IntSideDef* sd, int num);
	void WriteSectorUDMF(FWadWriter& out, IntSector* sec, int num);
	void WriteVertexUDMF(FWadWriter& out, IntVertex* vt, int num);
	void WriteTextMap(FWadWriter& out);

	char level_name[9];
	FLevel Level;

	TArray<FNodeBuilder::FPolyStart> poly_starts;
	TArray<FNodeBuilder::FPolyStart> poly_anchors;

	bool is_extended = false;
	bool is_udmf = false;
	znbx_NodeVersion node_version = ZNBX_NODEVERS_UNKNOWN;

	Scanner scanner;
	StringBuffer stbuf;
};

#endif //__PROCESSOR_H__
