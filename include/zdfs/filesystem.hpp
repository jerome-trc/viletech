/// @file
/// @brief File system I/O functions.

#pragma once

#include "files.hpp"
#include "resourcefile.hpp"
#include "zdfs.hpp"

namespace zdfs {

union LumpShortName {
	char String[9];
	uint32_t dword; // These are for accessing the first 4 or 8 chars of
	uint64_t qword; // Name as a unit without breaking strict aliasing rules
};

struct FolderEntry {
	const char *name;
	uint32_t lumpnum;
};

class FileSystem
{
public:
	FileSystem(FileSystemMessageFunc);
	~FileSystem();

	ZDFS_NODISCARD int GetIwadNum() const noexcept { return IwadIndex; }
	void SetIwadNum(int x) noexcept { IwadIndex = x; }

	ZDFS_NODISCARD int GetMaxIwadNum() const noexcept { return MaxIwadIndex; }
	void SetMaxIwadNum(int x) noexcept { MaxIwadIndex = x; }

	/// @throws FileSystemException if the given lump number is invalid.
	ZDFS_NODISCARD FileData entry_data(LumpNum) const;

	/// If the requested entry is absent, 0 will be returned (just as if the requested
	/// entry exists but has no flags set), but `exists` will be set to `false`.
	ZDFS_NODISCARD ELumpFlags entry_flags(LumpNum, bool& exists) const;

	/// Returns the entry's full name if it has one or its short name if not.
	/// Will return `NULL` if the given lump number is invalid.
	ZDFS_NODISCARD const char* entry_fullname(LumpNum, bool returnshort = true) const;

	/// Returns the buffer size needed to load the given lump.
	///
	/// If the requested entry is absent, 0 will be returned (just as if the requested
	/// entry exists but has length 0), but `exists` will be set to `false`.
	ZDFS_NODISCARD size_t entry_len(LumpNum, bool& exists) const;

	/// Will return `NULL` if the given lump number is invalid.
	ZDFS_NODISCARD const char* entry_shortname(LumpNum) const;

	/// Loads the lump into `dest`, which must be backed by a buffer >= the entry's length.
	/// @throws FileSystemException if the given lump number is invalid.
	/// @throws FileSystemException if not all expected bytes get read.
	void entry_read_into(LumpNum, void* dest) const;

	bool init_multiple_files(
		std::vector<std::string>& filenames,
		LumpFilterInfo* filter = nullptr,
		bool allowduplicates = false,
		FILE* hashfile = nullptr
	);

	ZDFS_NODISCARD size_t num_entries() const {
		return this->NumEntries;
	}

	ZDFS_NODISCARD size_t num_files() const {
		return this->Files.size();
	}

	bool init_single_file(const char* filename);

	void AddFile(
		const char *filename,
		FileReader *wadinfo,
		LumpFilterInfo* filter,
		FileSystemMessageFunc Printf,
		FILE* hashfile
	);

	int CheckIfResourceFileLoaded(const char* name) noexcept;

#if 0
	void AddAdditionalFile(const char* filename, FileReader* wadinfo = NULL) {}
#endif

	const char *GetResourceFileName (int filenum) const noexcept;
	const char *GetResourceFileFullName (int wadnum) const noexcept;

	ZDFS_NODISCARD LumpNum GetFirstEntry(int wadnum) const noexcept;
	ZDFS_NODISCARD LumpNum GetLastEntry(int wadnum) const noexcept;
    ZDFS_NODISCARD LumpNum GetEntryCount(int wadnum) const noexcept;

	ZDFS_NODISCARD LumpNum CheckNumForName(const char *name, int namespc) const;
	ZDFS_NODISCARD LumpNum CheckNumForName(const char *name, int namespc, int wadfile, bool exact = true) const;
	ZDFS_NODISCARD LumpNum GetNumForName(const char *name, int namespc) const;

	ZDFS_NODISCARD inline LumpNum CheckNumForName(const uint8_t *name) const {
		return CheckNumForName((const char *)name, ns_global);
	}

	ZDFS_NODISCARD inline LumpNum CheckNumForName(const char* name) const {
		return CheckNumForName(name, ns_global);
	}

	ZDFS_NODISCARD inline LumpNum CheckNumForName(const uint8_t* name, int ns) const {
		return CheckNumForName((const char*)name, ns);
	}

	ZDFS_NODISCARD inline LumpNum GetNumForName(const char* name) const {
		return GetNumForName(name, ns_global);
	}

	ZDFS_NODISCARD inline LumpNum GetNumForName(const uint8_t* name) const {
		return GetNumForName((const char*)name);
	}

	ZDFS_NODISCARD inline LumpNum GetNumForName(const uint8_t* name, int ns) const {
		return GetNumForName((const char*)name, ns);
	}

	ZDFS_NODISCARD int CheckNumForFullName(
		const char *cname,
		bool trynormal = false,
		int namespc = ns_global,
		bool ignoreext = false
	) const;

	int CheckNumForFullName(const char *name, int wadfile) const;

	int GetNumForFullName(const char *name) const;

	int FindFile(const char* name) const {
		return CheckNumForFullName(name);
	}

	bool FileExists(const char* name) const {
		return FindFile(name) >= 0;
	}

	bool FileExists(const std::string& name) const {
		return FindFile(name.c_str()) >= 0;
	}

	/// May only be called before the hash chains are set up.
	LumpShortName& GetShortName(LumpNum i);

	void RenameFile(LumpNum num, const char* fn);

	bool CreatePathlessCopy(const char* name, int id, int flags);

	// These should only be used if the file data really needs padding.
	FileData ReadFile(LumpNum lump);

	FileData ReadFile(const char *name) {
		return ReadFile(GetNumForName(name));
	}

	FileData ReadFileFullName(const char* name) {
		return ReadFile(GetNumForFullName(name));
	}

	// Opens a reader that redirects to that of the containing file.
	FileReader OpenFileReader(LumpNum lump, int readertype, int readerflags) const;

	FileReader OpenFileReader(const char* name);

	FileReader ReopenFileReader(const char* name, bool alwayscache = false);

	FileReader OpenFileReader(int lump) const {
		return OpenFileReader(lump, READER_SHARED, READERFLAG_SEEKABLE);
	}

	FileReader ReopenFileReader(int lump, bool alwayscache = false) {
		return OpenFileReader(lump, alwayscache ? READER_CACHED : READER_NEW, READERFLAG_SEEKABLE);
	}

	int FindLump (const char *name, int *lastlump, bool anyns=false);		// [RH] Find lumps with duplication
	int FindLumpMulti (const char **names, int *lastlump, bool anyns = false, int *nameindex = NULL); // same with multiple possible names
	int FindLumpFullName(const char* name, int* lastlump, bool noext = false);
	bool CheckFileName (int lump, const char *name);	// [RH] True if lump's name == name

	int FindFileWithExtensions(const char* name, const char* const* exts, int count) const;
	int FindResource(int resid, const char* type, int filenum = -1) const noexcept;
	int GetResource(int resid, const char* type, int filenum = -1) const;

	static uint32_t LumpNameHash (const char *name);		// [RH] Create hash key from an 8-char name

	std::string GetFileFullPath (int lump) const;		// [RH] Returns wad's name + lump's full name
	int GetFileContainer (int lump) const;				// [RH] Returns wadnum for a specified lump
	int GetFileNamespace (int lump) const;			// [RH] Returns the namespace a lump belongs to
	void SetFileNamespace(int lump, int ns);
	int GetResourceId(int lump) const;				// Returns the RFF index number for this lump
	const char* GetResourceType(int lump) const;
	bool CheckFileName (int lump, const char *name) const;	// [RH] Returns true if the names match
	unsigned GetFilesInFolder(const char *path, std::vector<FolderEntry> &result, bool atomic) const;

	int AddFromBuffer(const char* name, char* data, int size, int id, int flags);
	FileReader* GetFileReader(int wadnum);	// Gets a FileReader object to the entire WAD
	void init_hash_chains();

protected:

	struct LumpRecord;

	FileSystemMessageFunc Printf = nullptr;

	std::vector<FResourceFile *> Files;
	std::vector<LumpRecord> FileInfo;

	std::vector<uint32_t> Hashes;	// one allocation for all hash lists.
	uint32_t *FirstLumpIndex;	// [RH] Hashing stuff moved out of lumpinfo structure
	uint32_t *NextLumpIndex;

	uint32_t *FirstLumpIndex_FullName;	// The same information for fully qualified paths from .zips
	uint32_t *NextLumpIndex_FullName;

	uint32_t *FirstLumpIndex_NoExt;	// The same information for fully qualified paths from .zips
	uint32_t *NextLumpIndex_NoExt;

	uint32_t* FirstLumpIndex_ResId;	// The same information for fully qualified paths from .zips
	uint32_t* NextLumpIndex_ResId;

	uint32_t NumEntries = 0;					// Not necessarily the same as FileInfo.Size()
	uint32_t NumWads;

	int IwadIndex = -1;
	int MaxIwadIndex = -1;

	StringPool* stringpool = nullptr;

private:
	void DeleteAll();
	void MoveLumpsInFolder(const char *);

};

}
