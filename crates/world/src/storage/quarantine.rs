use crate::storage::{tile_container_path, tile_dir, WorldLayout};
use anyhow::Context;
use foundation::ids::TileId;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn quarantine_tile_dir(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
    reason: &str,
) -> anyhow::Result<PathBuf> {
    let source = tile_dir(layout, region, tile_id);
    let target = quarantine_path(layout, region, tile_id, "dir")?;
    fs::create_dir_all(
        target
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid quarantine path"))?,
    )
    .with_context(|| format!("create quarantine dir {:?}", target))?;
    if source.exists() {
        fs::rename(&source, &target)
            .with_context(|| format!("quarantine tile {:?}: {}", source, reason))?;
    }
    Ok(target)
}

pub fn quarantine_tile_file(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
    reason: &str,
) -> anyhow::Result<PathBuf> {
    let source = tile_container_path(layout, region, tile_id);
    let target = quarantine_path(layout, region, tile_id, "tile")?;
    fs::create_dir_all(
        target
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid quarantine path"))?,
    )
    .with_context(|| format!("create quarantine dir {:?}", target))?;
    if source.exists() {
        fs::rename(&source, &target)
            .with_context(|| format!("quarantine tile {:?}: {}", source, reason))?;
    }
    Ok(target)
}

fn quarantine_path(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
    suffix: &str,
) -> anyhow::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let target = layout
        .quarantine_dir
        .join(timestamp.to_string())
        .join(region)
        .join(format!(
            "x{}_y{}.{}",
            tile_id.coord.x, tile_id.coord.y, suffix
        ));
    Ok(target)
}
