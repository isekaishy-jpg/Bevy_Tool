use anyhow::bail;

#[derive(Debug, Clone, PartialEq)]
pub struct HmapSection {
    pub width: u16,
    pub height: u16,
    pub samples: Vec<f32>,
}

const HMAP_VERSION: u16 = 1;

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
