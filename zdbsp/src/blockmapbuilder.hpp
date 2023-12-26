#include "doomdata.hpp"
#include "workdata.hpp"
#include "tarray.hpp"

class FBlockmapBuilder
{
public:
	FBlockmapBuilder (FLevel &level);
	WORD *GetBlockmap (int &size);

private:
	FLevel &Level;
	TArray<WORD> BlockMap;

	void BuildBlockmap ();
	void CreateUnpackedBlockmap (TArray<WORD> *blocks, int bmapwidth, int bmapheight);
	void CreatePackedBlockmap (TArray<WORD> *blocks, int bmapwidth, int bmapheight);
};
