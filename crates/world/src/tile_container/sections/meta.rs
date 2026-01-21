use anyhow::bail;
use foundation::ids::TileId;

#[derive(Debug, Clone, PartialEq)]
pub struct MetaSection {
    pub format_version: u32,
    pub tile_id: TileId,
    pub region_hash: u64,
    pub created_timestamp: u64,
}

const META_VERSION: u16 = 1;

pub fn encode_meta(meta: &MetaSection) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(&META_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&meta.tile_id.coord.x.to_le_bytes());
    out.extend_from_slice(&meta.tile_id.coord.y.to_le_bytes());
    out.extend_from_slice(&meta.region_hash.to_le_bytes());
    out.extend_from_slice(&meta.format_version.to_le_bytes());
    out.extend_from_slice(&meta.created_timestamp.to_le_bytes());
    out
}

pub fn decode_meta(bytes: &[u8]) -> anyhow::Result<MetaSection> {
    if bytes.len() < 32 {
        bail!("META section too small");
    }
    let version = u16::from_le_bytes(bytes[0..2].try_into()?);
    if version != META_VERSION {
        bail!("unsupported META version {}", version);
    }
    let tile_x = i32::from_le_bytes(bytes[4..8].try_into()?);
    let tile_y = i32::from_le_bytes(bytes[8..12].try_into()?);
    let region_hash = u64::from_le_bytes(bytes[12..20].try_into()?);
    let format_version = u32::from_le_bytes(bytes[20..24].try_into()?);
    let created_timestamp = u64::from_le_bytes(bytes[24..32].try_into()?);
    Ok(MetaSection {
        format_version,
        tile_id: TileId {
            coord: foundation::ids::TileCoord {
                x: tile_x,
                y: tile_y,
            },
        },
        region_hash,
        created_timestamp,
    })
}
