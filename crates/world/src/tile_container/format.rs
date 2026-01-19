use anyhow::{anyhow, bail};
use std::fmt;

pub const TILE_MAGIC: [u8; 4] = *b"TILE";
pub const CONTAINER_VERSION: u16 = 1;
pub const MIN_CONTAINER_VERSION: u16 = 1;
pub const ENDIAN_LITTLE: u16 = 1;
pub const HEADER_SIZE: usize = 128;
pub const DIR_ENTRY_SIZE: usize = 64;
pub const MAX_SECTION_COUNT: u32 = 256;
pub const DEFAULT_ALIGNMENT: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileSectionTag(pub u32);

impl TileSectionTag {
    pub const META: Self = Self::from_bytes(*b"META");
    pub const HMAP: Self = Self::from_bytes(*b"HMAP");
    pub const WMAP: Self = Self::from_bytes(*b"WMAP");
    pub const LIQD: Self = Self::from_bytes(*b"LIQD");
    pub const PROP: Self = Self::from_bytes(*b"PROP");
    pub const SPLN: Self = Self::from_bytes(*b"SPLN");
    pub const ADDX: Self = Self::from_bytes(*b"ADDX");

    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }

    pub fn as_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn as_string(self) -> String {
        let bytes = self.as_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }
}

impl fmt::Display for TileSectionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TileContainerHeader {
    pub container_version: u16,
    pub flags: u32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub region_hash: u64,
    pub world_spec_hash: u64,
    pub section_count: u32,
    pub section_dir_offset: u64,
    pub created_timestamp: u64,
}

impl TileContainerHeader {
    pub fn new(tile_x: i32, tile_y: i32, region_hash: u64, world_spec_hash: u64) -> Self {
        Self {
            container_version: CONTAINER_VERSION,
            flags: 0,
            tile_x,
            tile_y,
            region_hash,
            world_spec_hash,
            section_count: 0,
            section_dir_offset: HEADER_SIZE as u64,
            created_timestamp: current_timestamp(),
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&TILE_MAGIC);
        bytes[4..6].copy_from_slice(&self.container_version.to_le_bytes());
        bytes[6..8].copy_from_slice(&ENDIAN_LITTLE.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.flags.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.tile_x.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.tile_y.to_le_bytes());
        bytes[20..28].copy_from_slice(&self.region_hash.to_le_bytes());
        bytes[28..36].copy_from_slice(&self.world_spec_hash.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.section_count.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.section_dir_offset.to_le_bytes());
        bytes[48..56].copy_from_slice(&self.created_timestamp.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < HEADER_SIZE {
            bail!("header too small: {} bytes", bytes.len());
        }
        if bytes[0..4] != TILE_MAGIC {
            bail!("invalid magic");
        }
        let container_version = u16::from_le_bytes(bytes[4..6].try_into()?);
        let endianness = u16::from_le_bytes(bytes[6..8].try_into()?);
        if endianness != ENDIAN_LITTLE {
            bail!("unsupported endianness: {}", endianness);
        }
        let flags = u32::from_le_bytes(bytes[8..12].try_into()?);
        let tile_x = i32::from_le_bytes(bytes[12..16].try_into()?);
        let tile_y = i32::from_le_bytes(bytes[16..20].try_into()?);
        let region_hash = u64::from_le_bytes(bytes[20..28].try_into()?);
        let world_spec_hash = u64::from_le_bytes(bytes[28..36].try_into()?);
        let section_count = u32::from_le_bytes(bytes[36..40].try_into()?);
        let section_dir_offset = u64::from_le_bytes(bytes[40..48].try_into()?);
        let created_timestamp = u64::from_le_bytes(bytes[48..56].try_into()?);

        if section_count > MAX_SECTION_COUNT {
            return Err(anyhow!("section_count {} exceeds cap", section_count));
        }

        Ok(Self {
            container_version,
            flags,
            tile_x,
            tile_y,
            region_hash,
            world_spec_hash,
            section_count,
            section_dir_offset,
            created_timestamp,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TileSectionDirEntry {
    pub tag: TileSectionTag,
    pub section_version: u16,
    pub codec: u16,
    pub flags: u32,
    pub offset: u64,
    pub stored_len: u64,
    pub decoded_len: u64,
    pub crc32: u32,
}

impl TileSectionDirEntry {
    pub fn to_bytes(&self) -> [u8; DIR_ENTRY_SIZE] {
        let mut bytes = [0u8; DIR_ENTRY_SIZE];
        bytes[0..4].copy_from_slice(&self.tag.as_bytes());
        bytes[4..6].copy_from_slice(&self.section_version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.codec.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.flags.to_le_bytes());
        bytes[12..20].copy_from_slice(&self.offset.to_le_bytes());
        bytes[20..28].copy_from_slice(&self.stored_len.to_le_bytes());
        bytes[28..36].copy_from_slice(&self.decoded_len.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.crc32.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < DIR_ENTRY_SIZE {
            bail!("directory entry too small");
        }
        let tag = TileSectionTag(u32::from_le_bytes(bytes[0..4].try_into()?));
        let section_version = u16::from_le_bytes(bytes[4..6].try_into()?);
        let codec = u16::from_le_bytes(bytes[6..8].try_into()?);
        let flags = u32::from_le_bytes(bytes[8..12].try_into()?);
        let offset = u64::from_le_bytes(bytes[12..20].try_into()?);
        let stored_len = u64::from_le_bytes(bytes[20..28].try_into()?);
        let decoded_len = u64::from_le_bytes(bytes[28..36].try_into()?);
        let crc32 = u32::from_le_bytes(bytes[36..40].try_into()?);
        Ok(Self {
            tag,
            section_version,
            codec,
            flags,
            offset,
            stored_len,
            decoded_len,
            crc32,
        })
    }
}

pub fn alignment_padding(offset: u64, alignment: u64) -> u64 {
    let remainder = offset % alignment;
    if remainder == 0 {
        0
    } else {
        alignment - remainder
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
