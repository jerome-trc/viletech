//! # WadLoad
//!
//! Simple, pay-for-what-you-use I/O for Doom's [WAD] archive file format.
//!
//! [WAD]: https://doomwiki.org/wiki/WAD

#![doc(
    html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
    html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

use std::{
    io::{Read, Seek, SeekFrom},
    ops::Range,
};

use util::{read_id8, Id8};

/// Whether this WAD is the basis of a game, or a "mod".
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WadKind {
    /// "Internal WAD". See <https://doomwiki.org/wiki/IWAD>.
    IWad,
    /// "Patch WAD". See <https://doomwiki.org/wiki/PWAD>.
    PWad,
}

/// Checks if `reader` represents an entire valid WAD file.
/// `reader`'s position upon return is **unspecifed**.
///
/// This check is not a complete guarantee, since it only checks the header and
/// stream length; it is possible that a lump is not in sync with the directory,
/// for example, but this is too involved a check to be applied often.
pub fn validate<R: Read + Seek>(reader: &mut R) -> Result<(), Error> {
    validate_impl(reader).map(|_| ())
}

/// A stream that wraps a [`std::io::Read`] implementation and yields a
/// full list of the entries in the directory part of a WAD file's header.
#[derive(Debug)]
pub struct DirReader<R: Read + Seek> {
    reader: R,
    kind: WadKind,
    len: usize,
    current: usize,
    /// Only updated by [`DirReader`]; only used by [`Reader`].
    stream_pos: u64,
}

impl<R: Read + Seek> DirReader<R> {
    /// Note that `reader` can be at any position when passed.
    ///
    /// # Errors
    /// - [`Error::InvalidKind`] if the 4-byte magic number at the start of the
    /// header is not IWAD or PWAD (ASCII), or is not present in full.
    /// - [`Error::InvalidEntryCount`] if the `i32` after the magic number
    /// is negative, or not present in full.
    /// - [`Error::InvalidDirOffset`] if the `i32` after the entry count
    /// is negative, or not present in full.
    /// - [`Error::Oversize`] if the directory size derived from the entry count
    /// or the expected size of the data is too big to be addressable.
    pub fn new(mut reader: R) -> Result<Self, Error> {
        let header = validate_impl(&mut reader)?;

        Ok(Self {
            reader,
            kind: header.kind,
            len: header.lump_c as usize,
            current: 0,
            stream_pos: header.dir_offs,
        })
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes this directory reader, returning the underlying value.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Is this an IWAD or a PWAD?
    #[must_use]
    pub fn wad_kind(&self) -> WadKind {
        self.kind
    }

    /// Returns the number of lumps in this WAD,
    /// regardless of the iterator's current place.
    #[must_use]
    pub fn lump_count(&self) -> usize {
        self.len
    }
}

impl<R: Read + Seek> Iterator for DirReader<R> {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.len {
            return None;
        }

        let mut ebuf = [0; 16];

        match self.reader.read_exact(&mut ebuf) {
            Ok(()) => {}
            Err(err) => {
                return Some(Err(Error::Io {
                    source: err,
                    context: "directory iteration",
                }))
            }
        }

        let offs = i32::from_le_bytes([ebuf[0], ebuf[1], ebuf[2], ebuf[3]]);
        let size = i32::from_le_bytes([ebuf[4], ebuf[5], ebuf[6], ebuf[7]]);

        if offs < 0 || size < 0 {
            return Some(Err(Error::InvalidDirEntry(self.current)));
        }

        let offs = offs as usize;
        let size = size as usize;

        let name = read_id8([
            ebuf[8], ebuf[9], ebuf[10], ebuf[11], ebuf[12], ebuf[13], ebuf[14], ebuf[15],
        ])
        .unwrap_or_default();

        let span = offs..(offs + size);

        self.current += 1;
        self.stream_pos += DIR_ENTRY_SIZE as u64;

        Some(Ok(DirEntry { name, span }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.lump_count(), Some(self.lump_count()))
    }
}

impl<R: Read + Seek> ExactSizeIterator for DirReader<R> {}

/// Yielded from [`DirReader`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirEntry {
    pub name: Id8,
    /// Note that this is relative to the whole WAD, not the end of the header.
    pub span: Range<usize>,
}

/// A thin wrapper around [`DirReader`] that maps its output, returning both
/// [`DirEntry`] and a `Vec<u8>`.
#[derive(Debug)]
pub struct Reader<R: Read + Seek> {
    inner: DirReader<R>,
}

impl<R: Read + Seek> Reader<R> {
    /// See [`DirReader::new`] for caveats; this just wraps that function.
    pub fn new(reader: R) -> Result<Self, Error> {
        DirReader::new(reader).map(|r| Self { inner: r })
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_ref(&self) -> &R {
        self.inner.get_ref()
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    /// Consumes this reader, returning the underlying value.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.inner.reader
    }

    /// Is this an IWAD or a PWAD?
    #[must_use]
    pub fn wad_kind(&self) -> WadKind {
        self.inner.wad_kind()
    }

    /// Returns the number of lumps in this WAD,
    /// regardless of the iterator's current place.
    #[must_use]
    pub fn lump_count(&self) -> usize {
        self.inner.lump_count()
    }
}

impl<R: Read + Seek> Iterator for Reader<R> {
    type Item = Result<(DirEntry, Vec<u8>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| match result {
            Ok(entry) => {
                self.inner
                    .reader
                    .seek(SeekFrom::Start(entry.span.start as u64))
                    .expect("failed to walk back `Reader`");

                let mut buf = vec![0; entry.span.len()];

                match self.inner.reader.read_exact(&mut buf) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(Error::Io {
                            source: err,
                            context: "streaming read",
                        });
                    }
                };

                self.inner
                    .reader
                    .seek(SeekFrom::Start(self.inner.stream_pos))
                    .expect("failed to reset `Reader`");

                Ok((entry, buf))
            }
            Err(err) => Err(err),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.lump_count(), Some(self.lump_count()))
    }
}

impl<R: Read + Seek> ExactSizeIterator for Reader<R> {}

/// An entry in a WAD file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lump {
    name: Id8,
    bytes: Box<[u8]>,
}

impl Lump {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn into_inner(self) -> (Id8, Vec<u8>) {
        (self.name, self.bytes.into_vec())
    }
}

impl From<(DirEntry, Vec<u8>)> for Lump {
    fn from(value: (DirEntry, Vec<u8>)) -> Self {
        Self {
            name: value.0.name,
            bytes: value.1.into_boxed_slice(),
        }
    }
}

/// Things that can go wrong when reading or writing WADs.
#[derive(Debug)]
pub enum Error {
    Io {
        source: std::io::Error,
        context: &'static str,
    },
    /// Can be raised when trying to read a header.
    InvalidKind([u8; 4]),
    /// Can be raised when trying to read a header.
    InvalidEntryCount(i32),
    /// Can be raised when trying to read a header.
    InvalidDirOffset(i32),
    /// The contained index is that of a directory entry that could not be fully
    /// read, or had a negative offset or size.
    InvalidDirEntry(usize),
    /// Can be raised when trying to read a header.
    Oversize,
    /// The header prescribed `n` number of lumps and the directory is at a byte
    /// offset of `o`, but `(16 * n) + o` is past the length of readable data.
    DataMalformed(usize),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { source, context } => {
                write!(f, "IO error: {source} (during operation: {context})")
            }
            Error::InvalidKind(chars) => {
                write!(f, "invalid WAD magic number: {chars:?}")
            }
            Error::InvalidEntryCount(c) => {
                write!(f, "WAD header has invalid directory entry count: {c}")
            }
            Error::InvalidDirOffset(offs) => {
                write!(f, "WAD header has invalid directory offset: {offs}")
            }
            Error::InvalidDirEntry(index) => {
                write!(
                    f,
                    "WAD directory entry {index} has a negative lump size or offset"
                )
            }
            Error::Oversize => {
                write!(f, "WAD file is larger than prescribed by its header")
            }
            Error::DataMalformed(_) => {
                write!(f, "disparity between WAD size and header information")
            }
        }
    }
}

pub(crate) const DIR_ENTRY_SIZE: usize = 16;

pub(crate) struct Header {
    kind: WadKind,
    dir_offs: u64,
    lump_c: i32,
}

fn validate_impl<R: Read + Seek>(reader: &mut R) -> Result<Header, Error> {
    let mut hbuf = [0; 12];

    reader.read_exact(&mut hbuf).map_err(|err| Error::Io {
        source: err,
        context: "header read",
    })?;

    let kind = match &hbuf[0..4] {
        b"IWAD" => WadKind::IWad,
        b"PWAD" => WadKind::PWad,
        _ => return Err(Error::InvalidKind([hbuf[0], hbuf[1], hbuf[2], hbuf[3]])),
    };

    let lump_c = i32::from_le_bytes([hbuf[4], hbuf[5], hbuf[6], hbuf[7]]);

    if lump_c < 0 {
        return Err(Error::InvalidEntryCount(lump_c));
    }

    let dir_offs = i32::from_le_bytes([hbuf[8], hbuf[9], hbuf[10], hbuf[11]]);

    if dir_offs < 0 {
        return Err(Error::InvalidDirOffset(dir_offs));
    }

    let expected_dir_len = (lump_c as usize)
        .checked_mul(DIR_ENTRY_SIZE)
        .ok_or(Error::Oversize)?;

    let expected_data_len = (dir_offs as usize)
        .checked_add(expected_dir_len)
        .ok_or(Error::Oversize)?;

    match reader.seek(SeekFrom::End(0)) {
        Ok(pos) => {
            if pos != (expected_data_len as u64) {
                return Err(Error::DataMalformed(pos as usize));
            }
        }
        Err(err) => {
            return Err(Error::Io {
                source: err,
                context: "data length validation",
            })
        }
    };

    let pos = reader
        .seek(SeekFrom::Start(dir_offs as u64))
        .expect("a pre-validated seek failed unexpectedly");

    Ok(Header {
        kind,
        dir_offs: pos,
        lump_c,
    })
}

#[cfg(test)]
mod test {
    use std::{io::BufReader, path::Path};

    use super::*;

    #[test]
    fn smoke() {
        let sample = Path::new(env!("CARGO_WORKSPACE_DIR")).join("sample/freedoom1.wad");

        if !sample.exists() {
            panic!(
                "WadLoad smoke testing depends on `{}`.\
				It can be acquired from https://freedoom.github.io/",
                sample.display()
            );
        }

        let file = std::fs::File::open(sample).unwrap();
        let bufr = BufReader::new(file);
        let mut reader = Reader::new(bufr).unwrap();

        assert_eq!(reader.wad_kind(), WadKind::IWad);

        let e1m1 = reader.next().unwrap().unwrap();

        assert_eq!(e1m1.0.name.as_str(), "E1M1");
        assert!(e1m1.1.is_empty());

        let e1m1_things = reader.next().unwrap().unwrap();

        assert_eq!(e1m1_things.0.name.as_str(), "THINGS");
        assert_eq!(e1m1_things.1.len(), 2380);
        assert_eq!(
            &e1m1_things.1[..8],
            &[0xB0, 0x06, 0x40, 0x04, 0x0E, 0x01, 0xDf, 0x07]
        );
        assert_eq!(
            &e1m1_things.1[2372..],
            &[0xF0, 0x05, 0x5A, 0x00, 0xDD, 0x07, 0x01, 0x00]
        );

        let remainder = reader.map(|result| result.unwrap()).collect::<Vec<_>>();

        let fcgrate2 = remainder
            .iter()
            .find(|p| p.0.name.as_str() == "FCGRATE2")
            .unwrap();
        assert_eq!(fcgrate2.1.len(), 4096);
        assert_eq!(
            &fcgrate2.1[..8],
            &[0x68, 0x6C, 0x6E, 0x6E, 0x6E, 0x6E, 0x6E, 0x6E]
        );
        assert_eq!(
            &fcgrate2.1[4088..],
            &[0x6F, 0x6F, 0x6F, 0x05, 0x05, 0x6E, 0x68, 0x66]
        );

        let f_end = remainder.last().unwrap();
        assert_eq!(f_end.0.name.as_str(), "F_END");
        assert!(f_end.1.is_empty());
    }
}
