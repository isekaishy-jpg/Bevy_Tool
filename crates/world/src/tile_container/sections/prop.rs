use anyhow::bail;
use foundation::ids::{AssetId, InstanceId};

use super::{read_string, write_string};

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

const PROP_VERSION: u16 = 1;

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
