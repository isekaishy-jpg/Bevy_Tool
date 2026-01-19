use crate::storage::{tile_dir, Layout};
use anyhow::Context;
use foundation::ids::TileId;
use std::fs;
use std::path::PathBuf;

pub fn quarantine_tile_dir(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    reason: &str,
) -> anyhow::Result<PathBuf> {
    let source = tile_dir(layout, region, tile_id);
    let target = layout
        .quarantine_dir
        .join(region)
        .join(format!("{}_{}", tile_id.coord.x, tile_id.coord.y));
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
