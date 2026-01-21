use anyhow::{anyhow, bail};

pub(super) fn write_string(out: &mut Vec<u8>, value: &str) -> anyhow::Result<()> {
    let bytes = value.as_bytes();
    let len = u16::try_from(bytes.len()).map_err(|_| anyhow!("string too long"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

pub(super) fn read_string(bytes: &[u8], offset: usize) -> anyhow::Result<(String, usize)> {
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
