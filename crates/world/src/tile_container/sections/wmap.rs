use anyhow::bail;

#[derive(Debug, Clone, PartialEq)]
pub struct WmapSection {
    pub width: u16,
    pub height: u16,
    pub layers: u16,
    pub weights: Vec<u8>,
}

const WMAP_VERSION: u16 = 1;

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
