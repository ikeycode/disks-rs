// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! LUKS2 superblock support

use std::io::Read;

use crate::{Error, Kind, Superblock};
use log;
use zerocopy::*;

const MAGIC_LEN: usize = 6;
const LABEL_LEN: usize = 48;
const CHECKSUM_ALG_LEN: usize = 32;
const SALT_LEN: usize = 64;
const UUID_LEN: usize = 40;
const CHECKSUM_LEN: usize = 64;

/// Per the `cryptsetup` docs for dm-crypt backed LUKS2, header is at first byte.
#[derive(FromBytes, Unaligned, Debug)]
#[repr(C, packed)]
pub struct Luks2 {
    magic: [u8; MAGIC_LEN],
    version: U16<BigEndian>,
    hdr_size: U64<BigEndian>,
    seqid: U64<BigEndian>,
    label: [u8; LABEL_LEN],
    checksum_alg: [u8; CHECKSUM_ALG_LEN],
    salt: [u8; SALT_LEN],
    uuid: [u8; UUID_LEN],
    subsystem: [u8; LABEL_LEN],
    hdr_offset: U64<BigEndian>,
    padding: [u8; 184],
    csum: [u8; CHECKSUM_LEN],
    padding4096: [u8; 7 * 512],
}

struct Magic;

// LUKS2 and SKUL2 are the two valid magic fields. Guess BigEndian came later?
impl Magic {
    const LUKS2: [u8; MAGIC_LEN] = [b'L', b'U', b'K', b'S', 0xba, 0xbe];
    const SKUL2: [u8; MAGIC_LEN] = [b'S', b'K', b'U', b'L', 0xba, 0xbe];
}

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Luks2, Error> {
    let data = Luks2::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    match data.magic {
        Magic::LUKS2 | Magic::SKUL2 => {
            log::trace!(
                "valid magic field: UUID={} [volume label: \"{}\"]",
                data.uuid()?,
                data.label().unwrap_or_else(|_| "[invalid utf8]".into())
            );
            Ok(data)
        }
        _ => Err(Error::InvalidMagic),
    }
}

impl Superblock for Luks2 {
    fn kind(&self) -> Kind {
        Kind::LUKS2
    }

    /// NOTE: LUKS2 stores string UUID rather than 128-bit sequence..
    fn uuid(&self) -> Result<String, super::Error> {
        Ok(std::str::from_utf8(&self.uuid)?.trim_end_matches('\0').to_owned())
    }

    /// NOTE: Label is often empty, set in config instead...
    fn label(&self) -> Result<String, super::Error> {
        Ok(std::str::from_utf8(&self.label)?.trim_end_matches('\0').to_owned())
    }
}

#[cfg(test)]
mod tests {

    use crate::{luks2::from_reader, Superblock};
    use std::fs;

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/luks+ext4.img.zst").expect("cannot open luks2 img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = from_reader(&mut stream).expect("Cannot parse superblock");
        assert_eq!(sb.uuid().unwrap(), "be373cae-2bd1-4ad5-953f-3463b2e53e59");
        assert_eq!(sb.version.get(), 2);
    }
}
