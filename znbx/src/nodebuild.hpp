#include <math.h>

#include "doomdata.hpp"
#include "tarray.hpp"

#include "znbx.h"

struct FEventInfo {
	int Vertex;
	uint32_t FrontSeg;
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
	znbx_I16F16 x, y;
};

extern "C" {
int ClassifyLine2(znbx_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]);
#ifndef DISABLE_SSE
int ClassifyLineSSE1(
	znbx_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
int ClassifyLineSSE2(
	znbx_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
#ifdef BACKPATCH
#ifdef __GNUC__
int ClassifyLineBackpatch(
	znbx_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
) __attribute__((noinline));
#else
int __declspec(noinline) ClassifyLineBackpatch(
	znbx_NodeFxp& node, const FSimpleVert* v1, const FSimpleVert* v2, int sidev[2]
);
#endif
#endif
#endif
}

class FNodeBuilder {
	struct FPrivSeg {
		int v1, v2;
		uint32_t sidedef;
		int linedef;
		int frontsector;
		int backsector;
		uint32_t next;
		uint32_t nextforvert;
		uint32_t nextforvert2;
		int loopnum; // loop number for split avoidance (0 means splitting is okay)
		uint32_t partner; // seg on back side
		uint32_t storedseg; // seg # in the GL_SEGS lump
		znbx_Angle angle;
		znbx_I16F16 offset;

		int planenum;
		bool planefront;
		FPrivSeg* hashnext;
	};

	struct FPrivVert : FSimpleVert {
		uint32_t segs; // segs that use this vertex as v1
		uint32_t segs2; // segs that use this vertex as v2
		int index;
		int pad; // This structure must be 8-byte aligned.

		bool operator==(const FPrivVert& other) {
			return x == other.x && y == other.y;
		}
	};

	struct FSimpleLine {
		znbx_I16F16 x, y, dx, dy;
	};

	union USegPtr {
		uint32_t SegNum;
		FPrivSeg* SegPtr;
	};

	struct FSplitSharer {
		double Distance;
		uint32_t Seg;
		bool Forward;
	};

	// Like a blockmap, but for vertices instead of lines
	class FVertexMap {
	public:
		FVertexMap(FNodeBuilder& builder, znbx_I16F16 minx, znbx_I16F16 miny, znbx_I16F16 maxx, znbx_I16F16 maxy);
		~FVertexMap();

		int SelectVertexExact(FPrivVert& vert);
		int SelectVertexClose(FPrivVert& vert);

	private:
		FNodeBuilder& MyBuilder;
		TArray<int>* VertexGrid;

		znbx_I16F16 MinX, MinY, MaxX, MaxY;
		int BlocksWide, BlocksTall;

		enum {
			BLOCK_SHIFT = 8 + FRACBITS
		};
		enum {
			BLOCK_SIZE = 1 << BLOCK_SHIFT
		};

		int InsertVertex(FPrivVert& vert);
		inline int GetBlock(znbx_I16F16 x, znbx_I16F16 y) {
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
		znbx_I16F16 x, y;
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

	void GetVertices(znbx_VertexEx*& verts, int32_t& count);
	void GetNodes(
		znbx_NodeEx*& nodes,
		int32_t& nodeCount,
		znbx_SegEx*& segs,
		int32_t& segCount,
		znbx_SubsectorEx*& ssecs,
		int32_t& subCount
	);

	void GetGLNodes(
		znbx_NodeEx*& nodes,
		int32_t& nodeCount,
		znbx_SegGlEx*& segs,
		int32_t& segCount,
		znbx_SubsectorEx*& ssecs,
		int32_t& subCount
	);

	// < 0 : in front of line
	// == 0 : on line
	// > 0 : behind line

	static inline int PointOnSide(int x, int y, int x1, int y1, int dx, int dy);

private:
	FVertexMap* VertexMap;

	TArray<znbx_NodeFxp> Nodes;
	TArray<znbx_SubsectorEx> Subsectors;
	TArray<uint32_t> SubsectorSets;
	TArray<FPrivSeg> Segs;
	TArray<FPrivVert> Vertices;
	TArray<USegPtr> SegList;
	TArray<uint8_t> PlaneChecked;
	TArray<FSimpleLine> Planes;
	size_t InitialVertices; // Number of vertices in a map that are connected to linedefs

	TArray<int> Touched; // Loops a splitter touches on a vertex
	TArray<int> Colinear; // Loops with edges colinear to a splitter
	FEventTree Events; // Vertices intersected by the current splitter
	TArray<uint32_t> UnsetSegs; // Segs with no definitive side in current splitter
	TArray<FSplitSharer> SplitSharers; // Segs collinear with the current splitter

	uint32_t HackSeg; // Seg to force to back of splitter
	uint32_t HackMate; // Seg to use in front of hack seg
	FLevel& Level;
	bool GLNodes;

	// Progress meter stuff
	int SegsStuffed;
	const char* MapName;

	void FindUsedVertices(znbx_VertexEx* vertices, int max);
	void BuildTree();
	void MakeSegsFromSides();
	int CreateSeg(int linenum, int sidenum);
	void GroupSegPlanes();
	void FindPolyContainers(TArray<FPolyStart>& spots, TArray<FPolyStart>& anchors);
	bool GetPolyExtents(int polynum, znbx_I16F16 bbox[4]);
	int MarkLoop(uint32_t firstseg, int loopnum);
	void AddSegToBBox(znbx_I16F16 bbox[4], const FPrivSeg* seg);
	uint32_t CreateNode(uint32_t set, unsigned int count, znbx_I16F16 bbox[4]);
	uint32_t CreateSubsector(uint32_t set, znbx_I16F16 bbox[4]);
	void CreateSubsectorsForReal();
	bool CheckSubsector(uint32_t set, znbx_NodeFxp& node, uint32_t& splitseg);
	bool CheckSubsectorOverlappingSegs(uint32_t set, znbx_NodeFxp& node, uint32_t& splitseg);
	void DoGLSegSplit(
		uint32_t set,
		znbx_NodeFxp& node,
		uint32_t splitseg,
		uint32_t& outset0,
		uint32_t& outset1,
		int side,
		int sidev0,
		int sidev1,
		bool hack
	);
	bool ShoveSegBehind(uint32_t set, znbx_NodeFxp& node, uint32_t seg, uint32_t mate);
	int SelectSplitter(uint32_t set, znbx_NodeFxp& node, uint32_t& splitseg, int step, bool nosplit);
	void SplitSegs(
		uint32_t set,
		znbx_NodeFxp& node,
		uint32_t splitseg,
		uint32_t& outset0,
		uint32_t& outset1,
		unsigned int& count0,
		unsigned int& count1
	);
	uint32_t SplitSeg(uint32_t segnum, int splitvert, int v1InFront);
	int Heuristic(znbx_NodeFxp& node, uint32_t set, bool honorNoSplit);

	// Returns:
	//	0 = seg is in front
	//  1 = seg is in back
	// -1 = seg cuts the node

	inline int ClassifyLine(
		znbx_NodeFxp& node, const FPrivVert* v1, const FPrivVert* v2, int sidev[2]
	);

	void FixSplitSharers(const znbx_NodeFxp& node);
	double AddIntersection(const znbx_NodeFxp& node, int vertex);
	void AddMinisegs(const znbx_NodeFxp& node, uint32_t splitseg, uint32_t& fset, uint32_t& rset);
	uint32_t CheckLoopStart(znbx_I16F16 dx, znbx_I16F16 dy, int vertex1, int vertex2);
	uint32_t CheckLoopEnd(znbx_I16F16 dx, znbx_I16F16 dy, int vertex2);
	void RemoveSegFromVert1(uint32_t segnum, int vertnum);
	void RemoveSegFromVert2(uint32_t segnum, int vertnum);
	uint32_t AddMiniseg(int v1, int v2, uint32_t partner, uint32_t seg1, uint32_t splitseg);
	void SetNodeFromSeg(znbx_NodeFxp& node, const FPrivSeg* pseg) const;

	int RemoveMinisegs(
		znbx_NodeEx* nodes,
		TArray<znbx_SegEx>& segs,
		znbx_SubsectorEx* subs,
		int node,
		short bbox[4]
	);
	int StripMinisegs(TArray<znbx_SegEx>& segs, int subsector, short bbox[4]);
	void AddSegToShortBBox(short bbox[4], const FPrivSeg* seg);
	int CloseSubsector(TArray<znbx_SegGlEx>& segs, int subsector);
	uint32_t PushGLSeg(TArray<znbx_SegGlEx>& segs, const FPrivSeg* seg);
	void PushConnectingGLSeg(int subsector, TArray<znbx_SegGlEx>& segs, int v1, int v2);
	int OutputDegenerateSubsector(
		TArray<znbx_SegGlEx>& segs, int subsector, bool bForward, double lastdot, FPrivSeg*& prev
	);

	static int SortSegs(const void* a, const void* b);

	double InterceptVector(const znbx_NodeFxp& splitter, const FPrivSeg& seg);

	void PrintSet(int l, uint32_t set);
	void DumpNodes(znbx_NodeEx* outNodes, int nodeCount);
};

// Points within this distance of a line will be considered on the line.
// Units are in znbx_I16F16s.
const double SIDE_EPSILON = 6.5536;

/// Vertices within this distance of each other will be considered as the same vertex.
/// Note that this is i16f16.
#define VERTEX_EPSILON 6

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
	znbx_NodeFxp& node, const FPrivVert* v1, const FPrivVert* v2, int sidev[2]
) {
	return ClassifyLine2(node, v1, v2, sidev);
}
