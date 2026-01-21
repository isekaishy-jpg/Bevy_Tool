use anyhow::{anyhow, bail};
use foundation::ids::{AssetId, InstanceId, TileId};

#[derive(Debug, Clone, PartialEq)]
pub struct MetaSection {
    pub format_version: u32,
    pub tile_id: TileId,
    pub region_hash: u64,
    pub created_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HmapSection {
    pub width: u16,
    pub height: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WmapSection {
    pub width: u16,
    pub height: u16,
    pub layers: u16,
    pub weights: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiqdSection {
    pub width: u16,
    pub height: u16,
    pub mask: Vec<u8>,
    pub bodies: Vec<LiqdBody>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiqdBody {
    pub id: u32,
    pub height: f32,
    pub kind: LiqdKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiqdKind {
    Water,
    Lava,
    Slime,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropSection {
    pub instances: Vec<PropRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropRecord {
    pub id: InstanceId,
    pub asset: AssetId,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

const META_VERSION: u16 = 1;
const HMAP_VERSION: u16 = 1;
const WMAP_VERSION: u16 = 1;
const LIQD_VERSION: u16 = 1;
const PROP_VERSION: u16 = 1;

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

pub fn encode_hmap(hmap: &HmapSection) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + hmap.samples.len() * 4);
    out.extend_from_slice(&HMAP_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&hmap.width.to_le_bytes());
    out.extend_from_slice(&hmap.height.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // encoding: f32
    out.extend_from_slice(&0u16.to_le_bytes());
    for sample in &hmap.samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

pub fn decode_hmap(bytes: &[u8]) -> anyhow::Result<HmapSection> {
    if bytes.len() < 12 {
        bail!("HMAP section too small");
    }
    let version = u16::from_le_bytes(bytes[0..2].try_into()?);
    if version != HMAP_VERSION {
        bail!("unsupported HMAP version {}", version);
    }
    let width = u16::from_le_bytes(bytes[4..6].try_into()?);
    let height = u16::from_le_bytes(bytes[6..8].try_into()?);
    let expected = width as usize * height as usize;
    let data_offset = 12;
    let mut samples = Vec::with_capacity(expected);
    let mut cursor = data_offset;
    while cursor + 4 <= bytes.len() && samples.len() < expected {
        let sample = f32::from_le_bytes(bytes[cursor..cursor + 4].try_into()?);
        samples.push(sample);
        cursor += 4;
    }
    if samples.len() != expected {
        bail!("HMAP sample count mismatch");
    }
    Ok(HmapSection {
        width,
        height,
        samples,
    })
}

pub fn encode_wmap(wmap: &WmapSection) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + wmap.weights.len());
    out.extend_from_slice(&WMAP_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&wmap.width.to_le_bytes());
    out.extend_from_slice(&wmap.height.to_le_bytes());
    out.extend_from_slice(&wmap.layers.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&wmap.weights);
    out
}

pub fn decode_wmap(bytes: &[u8]) -> anyhow::Result<WmapSection> {
    if bytes.len() < 12 {
        bail!("WMAP section too small");
    }
    let version = u16::from_le_bytes(bytes[0..2].try_into()?);
    if version != WMAP_VERSION {
        bail!("unsupported WMAP version {}", version);
    }
    let width = u16::from_le_bytes(bytes[4..6].try_into()?);
    let height = u16::from_le_bytes(bytes[6..8].try_into()?);
    let layers = u16::from_le_bytes(bytes[8..10].try_into()?);
    let expected = width as usize * height as usize * layers as usize;
    let weights = bytes[12..].to_vec();
    if weights.len() != expected {
        bail!("WMAP weight count mismatch");
    }
    Ok(WmapSection {
        width,
        height,
        layers,
        weights,
    })
}

pub fn encode_liqd(liqd: &LiqdSection) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + liqd.mask.len() + liqd.bodies.len() * 12);
    out.extend_from_slice(&LIQD_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&liqd.width.to_le_bytes());
    out.extend_from_slice(&liqd.height.to_le_bytes());
    out.extend_from_slice(&(liqd.bodies.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&liqd.mask);
    for body in &liqd.bodies {
        out.extend_from_slice(&body.id.to_le_bytes());
        out.extend_from_slice(&body.height.to_le_bytes());
        out.extend_from_slice(&encode_liqd_kind(&body.kind).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out
}

pub fn decode_liqd(bytes: &[u8]) -> anyhow::Result<LiqdSection> {
    if bytes.len() < 12 {
        bail!("LIQD section too small");
    }
    let version = u16::from_le_bytes(bytes[0..2].try_into()?);
    if version != LIQD_VERSION {
        bail!("unsupported LIQD version {}", version);
    }
    let width = u16::from_le_bytes(bytes[4..6].try_into()?);
    let height = u16::from_le_bytes(bytes[6..8].try_into()?);
    let body_count = u16::from_le_bytes(bytes[8..10].try_into()?);
    let mask_len = width as usize * height as usize;
    let mut cursor = 12;
    if bytes.len() < cursor + mask_len {
        bail!("LIQD mask truncated");
    }
    let mask = bytes[cursor..cursor + mask_len].to_vec();
    cursor += mask_len;
    let mut bodies = Vec::with_capacity(body_count as usize);
    for _ in 0..body_count {
        if bytes.len() < cursor + 12 {
            bail!("LIQD body truncated");
        }
        let id = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into()?);
        let height = f32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into()?);
        let kind_raw = u16::from_le_bytes(bytes[cursor + 8..cursor + 10].try_into()?);
        let kind = decode_liqd_kind(kind_raw)?;
        bodies.push(LiqdBody { id, height, kind });
        cursor += 12;
    }
    Ok(LiqdSection {
        width,
        height,
        mask,
        bodies,
    })
}

pub fn encode_prop(prop: &PropSection) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(&PROP_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(prop.instances.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    let mut instances = prop.instances.clone();
    instances.sort_by_key(|instance| instance.id.0);
    for instance in &instances {
        out.extend_from_slice(&instance.id.0.to_le_bytes());
        write_string(&mut out, &instance.asset.namespace)?;
        write_string(&mut out, &instance.asset.name)?;
        for value in instance.translation {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for value in instance.rotation {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for value in instance.scale {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(out)
}

pub fn decode_prop(bytes: &[u8]) -> anyhow::Result<PropSection> {
    if bytes.len() < 12 {
        bail!("PROP section too small");
    }
    let version = u16::from_le_bytes(bytes[0..2].try_into()?);
    if version != PROP_VERSION {
        bail!("unsupported PROP version {}", version);
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into()?);
    let mut cursor = 12;
    let mut instances = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if bytes.len() < cursor + 8 {
            bail!("PROP record truncated");
        }
        let id = InstanceId(u64::from_le_bytes(bytes[cursor..cursor + 8].try_into()?));
        cursor += 8;
        let (namespace, ns_len) = read_string(bytes, cursor)?;
        cursor += ns_len;
        let (name, name_len) = read_string(bytes, cursor)?;
        cursor += name_len;
        if bytes.len() < cursor + 40 {
            bail!("PROP transform truncated");
        }
        let translation = [
            f32::from_le_bytes(bytes[cursor..cursor + 4].try_into()?),
            f32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into()?),
            f32::from_le_bytes(bytes[cursor + 8..cursor + 12].try_into()?),
        ];
        let rotation = [
            f32::from_le_bytes(bytes[cursor + 12..cursor + 16].try_into()?),
            f32::from_le_bytes(bytes[cursor + 16..cursor + 20].try_into()?),
            f32::from_le_bytes(bytes[cursor + 20..cursor + 24].try_into()?),
            f32::from_le_bytes(bytes[cursor + 24..cursor + 28].try_into()?),
        ];
        let scale = [
            f32::from_le_bytes(bytes[cursor + 28..cursor + 32].try_into()?),
            f32::from_le_bytes(bytes[cursor + 32..cursor + 36].try_into()?),
            f32::from_le_bytes(bytes[cursor + 36..cursor + 40].try_into()?),
        ];
        cursor += 40;
        instances.push(PropRecord {
            id,
            asset: AssetId { namespace, name },
            translation,
            rotation,
            scale,
        });
    }
    Ok(PropSection { instances })
}

// NOTE: Custom liquid names are not serialized yet; only a sentinel is stored.
// Keep this behavior until the liquids schema checklist defines a stable encoding.
fn encode_liqd_kind(kind: &LiqdKind) -> u16 {
    match kind {
        LiqdKind::Water => 0,
        LiqdKind::Lava => 1,
        LiqdKind::Slime => 2,
        LiqdKind::Custom(_) => 255,
    }
}

fn decode_liqd_kind(raw: u16) -> anyhow::Result<LiqdKind> {
    match raw {
        0 => Ok(LiqdKind::Water),
        1 => Ok(LiqdKind::Lava),
        2 => Ok(LiqdKind::Slime),
        255 => Ok(LiqdKind::Custom("custom".to_string())),
        other => Err(anyhow!("unknown LIQD kind {}", other)),
    }
}

fn write_string(out: &mut Vec<u8>, value: &str) -> anyhow::Result<()> {
    let bytes = value.as_bytes();
    let len = u16::try_from(bytes.len()).map_err(|_| anyhow!("string too long"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_string(bytes: &[u8], offset: usize) -> anyhow::Result<(String, usize)> {
    if bytes.len() < offset + 2 {
        bail!("string length truncated");
    }
    let len = u16::from_le_bytes(bytes[offset..offset + 2].try_into()?) as usize;
    let start = offset + 2;
    let end = start + len;
    if bytes.len() < end {
        bail!("string bytes truncated");
    }
    let value = std::str::from_utf8(&bytes[start..end])?.to_string();
    Ok((value, 2 + len))
}
