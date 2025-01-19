// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Superblock detection for various filesystems

use std::io::{self, Read, Seek};

use thiserror::Error;

pub mod btrfs;
pub mod ext4;
pub mod f2fs;
pub mod luks2;
pub mod xfs;

/// Supported list of superblock readers
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Btrfs,
    Ext4,
    LUKS2,
    F2FS,
    XFS,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Kind::Btrfs => f.write_str("btrfs"),
            Kind::Ext4 => f.write_str("ext4"),
            Kind::LUKS2 => f.write_str("luks2"),
            Kind::F2FS => f.write_str("f2fs"),
            Kind::XFS => f.write_str("xfs"),
        }
    }
}

pub trait Superblock: std::fmt::Debug + Sync + Send {
    /// Return the superblock's kind
    fn kind(&self) -> self::Kind;

    /// Get the filesystem UUID
    fn uuid(&self) -> Result<String, self::Error>;

    /// Get the volume label
    fn label(&self) -> Result<String, self::Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown superblock")]
    UnknownSuperblock,

    #[error("decoding wrong superblock")]
    InvalidSuperblock,

    // ie label requests on partially implemented superblocks
    #[error("unsupported feature")]
    UnsupportedFeature,

    #[error("invalid utf8 in decode: {0}")]
    Utf8Decoding(#[from] std::str::Utf8Error),

    #[error("invalid utf16 in decode: {0}")]
    Utf16Decoding(#[from] std::string::FromUtf16Error),

    #[error("invalid magic in superblock")]
    InvalidMagic,

    #[error("io: {0}")]
    IO(#[from] io::Error),
}

/// Attempt to find a superblock decoder for the given reader
pub fn for_reader<R: Read + Seek>(reader: &mut R) -> Result<Box<dyn Superblock>, Error> {
    reader.rewind()?;

    // try ext4
    if let Ok(block) = ext4::from_reader(reader) {
        return Ok(Box::new(block));
    }

    // try btrfs
    reader.rewind()?;
    if let Ok(block) = btrfs::from_reader(reader) {
        return Ok(Box::new(block));
    }

    // try f2fs
    reader.rewind()?;
    if let Ok(block) = f2fs::from_reader(reader) {
        return Ok(Box::new(block));
    }

    // try xfs
    reader.rewind()?;
    if let Ok(block) = xfs::from_reader(reader) {
        return Ok(Box::new(block));
    }

    // try luks2
    reader.rewind()?;
    if let Ok(block) = luks2::from_reader(reader) {
        return Ok(Box::new(block));
    }

    Err(Error::UnknownSuperblock)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Read},
    };

    use crate::Kind;

    use super::for_reader;

    #[test_log::test]
    fn test_determination() {
        let tests = vec![
            ("btrfs", Kind::Btrfs),
            ("ext4", Kind::Ext4),
            ("f2fs", Kind::F2FS),
            ("luks+ext4", Kind::LUKS2),
            ("xfs", Kind::XFS),
        ];

        // Pre-allocate a buffer for determination tests
        let mut memory: Vec<u8> = Vec::with_capacity(6 * 1024 * 1024);

        for (fsname, _kind) in tests.into_iter() {
            // Swings and roundabouts: Unpack ztd image in memory to get the Seekable trait we need
            // While each Superblock API is non-seekable, we enforce superblock::for_reader to be seekable
            // to make sure we pre-read a blob and pass it in for rewind/speed.
            memory.clear();

            let mut fi = fs::File::open(format!("tests/{fsname}.img.zst")).expect("Cannot find test image");
            let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
            stream
                .read_to_end(&mut memory)
                .expect("Could not unpack ext4 filesystem in memory");

            let mut cursor = Cursor::new(&mut memory);
            let block = for_reader(&mut cursor).expect("Failed to find right block implementation");
            eprintln!("{fsname}.img.zstd: superblock matched to {}", block.kind());
            assert!(matches!(block.kind(), _kind));
        }
    }
}
