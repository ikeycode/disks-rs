// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! XFS superblock handling

use crate::{Error, Kind, Superblock};
use std::io::Read;
use uuid::Uuid;
use zerocopy::*;

// XFS typedefs
type RfsBlock = U64<BigEndian>;
type RtbXlen = U64<BigEndian>;
type FsBlock = U64<BigEndian>;
type Ino = I64<BigEndian>;
type AgBlock = U32<BigEndian>;
type AgCount = U32<BigEndian>;
type ExtLen = U32<BigEndian>;
type Lsn = I64<BigEndian>;

const MAX_LABEL_LEN: usize = 12;

/// XFS superblock, aligned to 64-bit
/// Note: Multi-byte integers (>{i,u}8) must be read as Big Endian
#[derive(FromBytes, Debug)]
#[repr(C, align(8))]
pub struct XFS {
    magicnum: U32<BigEndian>,
    blocksize: U32<BigEndian>,
    dblocks: RfsBlock,
    rblocks: RfsBlock,
    rextents: RtbXlen,
    uuid: [u8; 16],
    logstart: FsBlock,
    rootino: Ino,
    rbmino: Ino,
    rsumino: Ino,
    rextsize: AgBlock,
    agblocks: AgBlock,
    agcount: AgCount,
    rbmblocks: ExtLen,
    logblocks: ExtLen,
    versionnum: U16<BigEndian>,
    sectsize: U16<BigEndian>,
    inodesize: U16<BigEndian>,
    inopblock: U16<BigEndian>,
    fname: [u8; MAX_LABEL_LEN],
    blocklog: u8,
    sectlog: u8,
    inodelog: u8,
    inopblog: u8,
    agblklog: u8,
    rextslog: u8,
    inprogress: u8,
    imax_pct: u8,

    icount: U64<BigEndian>,
    ifree: U64<BigEndian>,
    fdblocks: U64<BigEndian>,
    frextents: U64<BigEndian>,

    uquotino: Ino,
    gquotino: Ino,
    qflags: U16<BigEndian>,
    flags: u8,
    shared_vn: u8,
    inoalignment: ExtLen,
    unit: U32<BigEndian>,
    width: U32<BigEndian>,
    dirblklog: u8,
    logsectlog: u8,
    logsectsize: U16<BigEndian>,
    logsunit: U32<BigEndian>,
    features2: U32<BigEndian>,

    bad_features: U32<BigEndian>,

    features_compat: U32<BigEndian>,
    features_ro_cmopat: U32<BigEndian>,
    features_incompat: U32<BigEndian>,
    features_log_incompat: U32<BigEndian>,

    crc: U32<BigEndian>,
    spino_align: ExtLen,

    pquotino: Ino,
    lsn: Lsn,
    meta_uuid: [u8; 16],
}

/// Magic = 'XFSB'
const MAGIC: U32<BigEndian> = U32::new(0x58465342);

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<XFS, Error> {
    let data = XFS::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    if data.magicnum != MAGIC {
        Err(Error::InvalidMagic)
    } else {
        log::trace!(
            "valid magic field: UUID={} [volume label: \"{}\"]",
            data.uuid()?,
            data.label().unwrap_or_else(|_| "[invalid utf8]".into())
        );
        Ok(data)
    }
}

impl Superblock for XFS {
    fn kind(&self) -> Kind {
        Kind::XFS
    }

    /// Return `uuid` as a properly formatted 128-bit UUID
    fn uuid(&self) -> Result<String, super::Error> {
        Ok(Uuid::from_bytes(self.uuid).hyphenated().to_string())
    }

    /// Return `fname` (volume name) as utf8 string
    fn label(&self) -> Result<String, super::Error> {
        Ok(std::str::from_utf8(&self.fname)?.trim_end_matches('\0').to_owned())
    }
}

#[cfg(test)]
mod tests {

    use crate::{xfs::from_reader, Superblock};
    use std::fs;

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/xfs.img.zst").expect("cannot open xfs img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = from_reader(&mut stream).expect("Cannot parse superblock");
        let label = sb.label().expect("Cannot determine volume name");
        assert_eq!(label, "BLSFORME");
        assert_eq!(sb.uuid().unwrap(), "45e8a3bf-8114-400f-95b0-380d0fb7d42d");
        assert_eq!(sb.versionnum, 46245);
    }
}
