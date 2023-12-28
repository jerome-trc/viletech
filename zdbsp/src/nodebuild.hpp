#include <math.h>
#include "doomdata.hpp"
#include "tarray.hpp"
#include "zdbsp.h"

struct FEventInfo {
	int Vertex;
	DWORD FrontSeg;
};

struct FEvent {
	FEvent *Parent, *Left, *Right;
	double Distance;
	FEventInfo Info;
};

class FEventTree {
public:
	FEventTree();
	~FEventTree();

	FEvent* GetMinimum();
	FEvent* GetSuccessor(FEvent* event) const {
		FEvent* node = Successor(event);
		return node == &Nil ? NULL : node;
	}
	FEvent* GetPredecessor(FEvent* event) const {
		FEvent* node = Predecessor(event);
		return node == &Nil ? NULL : node;
	}

	FEvent* GetNewNode();
	void Insert(FEvent* event);
	FEvent* FindEvent(double distance) const;
	void DeleteAll();

	void PrintTree() const {
		PrintTree(Root);
	}

private:
	FEvent Nil;
	FEvent* Root;
	FEvent* Spare;

	void DeletionTraverser(FEvent* event);
	FEvent* Successor(FEvent* event) const;
	FEvent* Predecessor(FEvent* event) const;

	void PrintTree(const FEvent* event) const;
};

struct FSimpleVert {
	fixed_t x, y;
};

extern "C" {
int ClassifyLine2(zdbsp_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]);
#ifndef DISABLE_SSE
int ClassifyLineSSE1(
	zdbsp_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
int ClassifyLineSSE2(
	zdbsp_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
#ifdef BACKPATCH
#ifdef __GNUC__
int ClassifyLineBackpatch(
	zdbsp_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
) __attribute__((noinline));
#else
int __declspec(noinline) ClassifyLineBackpatch(
	zdbsp_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
#endif
#endif
#endif
}

class FNodeBuilder {
	struct FPrivSeg {
		int v1, v2;
		DWORD sidedef;
		int linedef;
		int frontsector;
		int backsector;
		DWORD next;
		DWORD nextforvert;
		DWORD nextforvert2;
		int loopnum; // loop number for split avoidance (0 means splitting is okay)
		DWORD partner; // seg on back side
		DWORD storedseg; // seg # in the GL_SEGS lump
		angle_t angle;
		fixed_t offset;

		int planenum;
		bool planefront;
		FPrivSeg* hashnext;
	};

	struct FPrivVert : FSimpleVert {
		DWORD segs; // segs that use this vertex as v1
		DWORD segs2; // segs that use this vertex as v2
		int index;
		int pad; // This structure must be 8-byte aligned.

		bool operator==(const FPrivVert& other) {
			return x == other.x && y == other.y;
		}
	};

	struct FSimpleLine {
		fixed_t x, y, dx, dy;
	};

	union USegPtr {
		DWORD SegNum;
		FPrivSeg* SegPtr;
	};

	struct FSplitSharer {
		double Distance;
		DWORD Seg;
		bool Forward;
	};

	// Like a blockmap, but for vertices instead of lines
	class FVertexMap {
	public:
		FVertexMap(FNodeBuilder& builder, fixed_t minx, fixed_t miny, fixed_t maxx, fixed_t maxy);
		~FVertexMap();

		int SelectVertexExact(FPrivVert& vert);
		int SelectVertexClose(FPrivVert& vert);

	private:
		FNodeBuilder& MyBuilder;
		TArray<int>* VertexGrid;

		fixed_t MinX, MinY, MaxX, MaxY;
		int BlocksWide, BlocksTall;

		enum {
			BLOCK_SHIFT = 8 + FRACBITS
		};
		enum {
			BLOCK_SIZE = 1 << BLOCK_SHIFT
		};

		int InsertVertex(FPrivVert& vert);
		inline int GetBlock(fixed_t x, fixed_t y) {
			assert(x >= MinX);
			assert(y >= MinY);
			assert(x <= MaxX);
			assert(y <= MaxY);
			return (unsigned(x - MinX) >> BLOCK_SHIFT) +
				   (unsigned(y - MinY) >> BLOCK_SHIFT) * BlocksWide;
		}
	};

	friend class FVertexMap;

public:
	struct FPolyStart {
		int polynum;
		fixed_t x, y;
	};

	int32_t max_segs = 64, split_cost = 8, aa_pref = 16;

	FNodeBuilder(
		FLevel& level,
		TArray<FPolyStart>& polyspots,
		TArray<FPolyStart>& anchors,
		const char* name,
		bool makeGLnodes
	);

	~FNodeBuilder();

	void GetVertices(zdbsp_VertexEx*& verts, int32_t& count);
	void GetNodes(
		zdbsp_NodeEx*& nodes,
		int32_t& nodeCount,
		zdbsp_SegEx*& segs,
		int32_t& segCount,
		zdbsp_SubsectorEx*& ssecs,
		int32_t& subCount
	);

	void GetGLNodes(
		zdbsp_NodeEx*& nodes,
		int32_t& nodeCount,
		zdbsp_SegGlEx*& segs,
		int32_t& segCount,
		zdbsp_SubsectorEx*& ssecs,
		int32_t& subCount
	);

	// < 0 : in front of line
	// == 0 : on line
	// > 0 : behind line

	static inline int PointOnSide(int x, int y, int x1, int y1, int dx, int dy);

private:
	FVertexMap* VertexMap;

	TArray<zdbsp_NodeFxp> Nodes;
	TArray<zdbsp_SubsectorEx> Subsectors;
	TArray<DWORD> SubsectorSets;
	TArray<FPrivSeg> Segs;
	TArray<FPrivVert> Vertices;
	TArray<USegPtr> SegList;
	TArray<BYTE> PlaneChecked;
	TArray<FSimpleLine> Planes;
	size_t InitialVertices; // Number of vertices in a map that are connected to linedefs

	TArray<int> Touched; // Loops a splitter touches on a vertex
	TArray<int> Colinear; // Loops with edges colinear to a splitter
	FEventTree Events; // Vertices intersected by the current splitter
	TArray<uint32_t> UnsetSegs; // Segs with no definitive side in current splitter
	TArray<FSplitSharer> SplitSharers; // Segs collinear with the current splitter

	DWORD HackSeg; // Seg to force to back of splitter
	DWORD HackMate; // Seg to use in front of hack seg
	FLevel& Level;
	bool GLNodes;

	// Progress meter stuff
	int SegsStuffed;
	const char* MapName;

	void FindUsedVertices(zdbsp_VertexEx* vertices, int max);
	void BuildTree();
	void MakeSegsFromSides();
	int CreateSeg(int linenum, int sidenum);
	void GroupSegPlanes();
	void FindPolyContainers(TArray<FPolyStart>& spots, TArray<FPolyStart>& anchors);
	bool GetPolyExtents(int polynum, fixed_t bbox[4]);
	int MarkLoop(DWORD firstseg, int loopnum);
	void AddSegToBBox(fixed_t bbox[4], const FPrivSeg* seg);
	DWORD CreateNode(DWORD set, unsigned int count, fixed_t bbox[4]);
	DWORD CreateSubsector(DWORD set, fixed_t bbox[4]);
	void CreateSubsectorsForReal();
	bool CheckSubsector(DWORD set, zdbsp_NodeFxp& node, DWORD& splitseg);
	bool CheckSubsectorOverlappingSegs(DWORD set, zdbsp_NodeFxp& node, DWORD& splitseg);
	void DoGLSegSplit(
		uint32_t set,
		zdbsp_NodeFxp& node,
		uint32_t splitseg,
		uint32_t& outset0,
		uint32_t& outset1,
		int side,
		int sidev0,
		int sidev1,
		bool hack
	);
	bool ShoveSegBehind(DWORD set, zdbsp_NodeFxp& node, DWORD seg, DWORD mate);
	int SelectSplitter(DWORD set, zdbsp_NodeFxp& node, DWORD& splitseg, int step, bool nosplit);
	void SplitSegs(
		DWORD set,
		zdbsp_NodeFxp& node,
		DWORD splitseg,
		DWORD& outset0,
		DWORD& outset1,
		unsigned int& count0,
		unsigned int& count1
	);
	DWORD SplitSeg(DWORD segnum, int splitvert, int v1InFront);
	int Heuristic(zdbsp_NodeFxp& node, DWORD set, bool honorNoSplit);

	// Returns:
	//	0 = seg is in front
	//  1 = seg is in back
	// -1 = seg cuts the node

	inline int ClassifyLine(
		zdbsp_NodeFxp& node, const FPrivVert* v1, const FPrivVert* v2, int sidev[2]
	);

	void FixSplitSharers(const zdbsp_NodeFxp& node);
	double AddIntersection(const zdbsp_NodeFxp& node, int vertex);
	void AddMinisegs(const zdbsp_NodeFxp& node, DWORD splitseg, DWORD& fset, DWORD& rset);
	DWORD CheckLoopStart(fixed_t dx, fixed_t dy, int vertex1, int vertex2);
	DWORD CheckLoopEnd(fixed_t dx, fixed_t dy, int vertex2);
	void RemoveSegFromVert1(DWORD segnum, int vertnum);
	void RemoveSegFromVert2(DWORD segnum, int vertnum);
	DWORD AddMiniseg(int v1, int v2, DWORD partner, DWORD seg1, DWORD splitseg);
	void SetNodeFromSeg(zdbsp_NodeFxp& node, const FPrivSeg* pseg) const;

	int RemoveMinisegs(
		zdbsp_NodeEx* nodes,
		TArray<zdbsp_SegEx>& segs,
		zdbsp_SubsectorEx* subs,
		int node,
		short bbox[4]
	);
	int StripMinisegs(TArray<zdbsp_SegEx>& segs, int subsector, short bbox[4]);
	void AddSegToShortBBox(short bbox[4], const FPrivSeg* seg);
	int CloseSubsector(TArray<zdbsp_SegGlEx>& segs, int subsector);
	DWORD PushGLSeg(TArray<zdbsp_SegGlEx>& segs, const FPrivSeg* seg);
	void PushConnectingGLSeg(int subsector, TArray<zdbsp_SegGlEx>& segs, int v1, int v2);
	int OutputDegenerateSubsector(
		TArray<zdbsp_SegGlEx>& segs, int subsector, bool bForward, double lastdot, FPrivSeg*& prev
	);

	static int SortSegs(const void* a, const void* b);

	double InterceptVector(const zdbsp_NodeFxp& splitter, const FPrivSeg& seg);

	void PrintSet(int l, DWORD set);
	void DumpNodes(zdbsp_NodeEx* outNodes, int nodeCount);
};

// Points within this distance of a line will be considered on the line.
// Units are in fixed_ts.
const double SIDE_EPSILON = 6.5536;

// Vertices within this distance of each other will be considered as the same vertex.
#define VERTEX_EPSILON 6 // This is a fixed_t value

inline int FNodeBuilder::PointOnSide(int x, int y, int x1, int y1, int dx, int dy) {
	// For most cases, a simple dot product is enough.
	double d_dx = double(dx);
	double d_dy = double(dy);
	double d_x = double(x);
	double d_y = double(y);
	double d_x1 = double(x1);
	double d_y1 = double(y1);

	double s_num = (d_y1 - d_y) * d_dx - (d_x1 - d_x) * d_dy;

	if (fabs(s_num) < 17179869184.f) // 4<<32
	{
		// Either the point is very near the line, or the segment defining
		// the line is very short: Do a more expensive test to determine
		// just how far from the line the point is.
		double l = d_dx * d_dx + d_dy * d_dy; // double l = sqrt(d_dx*d_dx+d_dy*d_dy);
		double dist = s_num * s_num / l; // double dist = fabs(s_num)/l;
		if (dist < SIDE_EPSILON * SIDE_EPSILON) // if (dist < SIDE_EPSILON)
		{
			return 0;
		}
	}
	return s_num > 0.0 ? -1 : 1;
}

inline int FNodeBuilder::ClassifyLine(
	zdbsp_NodeFxp& node, const FPrivVert* v1, const FPrivVert* v2, int sidev[2]
) {
	return ClassifyLine2(node, v1, v2, sidev);
}
