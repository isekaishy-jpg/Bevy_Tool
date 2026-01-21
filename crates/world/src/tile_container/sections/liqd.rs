use anyhow::{anyhow, bail};

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

const LIQD_VERSION: u16 = 1;

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
