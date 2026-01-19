use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{ensure_tile_dir, tile_dir, Layout};
use anyhow::Context;
use foundation::ids::TileId;
use serde::{Deserialize, Serialize};
use std::fs;

const TILE_META_FILE: &str = "tile.meta.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TileMeta {
    pub format_version: u32,
    pub tile_id: TileId,
}

impl TileMeta {
    pub fn new(tile_id: TileId) -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            tile_id,
        }
    }
}

pub fn write_tile_meta(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    meta: &TileMeta,
) -> anyhow::Result<()> {
    let dir = ensure_tile_dir(layout, region, tile_id)?;
    let path = dir.join(TILE_META_FILE);
    let bytes = serde_json::to_vec_pretty(meta)?;
    fs::write(&path, bytes).with_context(|| format!("write tile meta {:?}", path))?;
    Ok(())
}

pub fn read_tile_meta(layout: &Layout, region: &str, tile_id: TileId) -> anyhow::Result<TileMeta> {
    let path = tile_dir(layout, region, tile_id).join(TILE_META_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read tile meta {:?}", path))?;
    let meta = serde_json::from_slice(&bytes)?;
    Ok(meta)
}
