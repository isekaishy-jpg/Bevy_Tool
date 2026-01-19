use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{ensure_tile_dir, tile_dir, Layout};
use anyhow::Context;
use foundation::ids::TileId;
use serde::{Deserialize, Serialize};
use std::fs;

const TERRAIN_HEIGHT_FILE: &str = "terrain.height.bin";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerrainHeight {
    pub format_version: u32,
    pub size: u16,
    pub samples: Vec<f32>,
}

impl TerrainHeight {
    pub fn new(size: u16, samples: Vec<f32>) -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            size,
            samples,
        }
    }
}

pub fn write_terrain_height(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    height: &TerrainHeight,
) -> anyhow::Result<()> {
    let dir = ensure_tile_dir(layout, region, tile_id)?;
    let path = dir.join(TERRAIN_HEIGHT_FILE);
    let bytes = serde_json::to_vec_pretty(height)?;
    fs::write(&path, bytes).with_context(|| format!("write terrain height {:?}", path))?;
    Ok(())
}

pub fn read_terrain_height(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<TerrainHeight> {
    let path = tile_dir(layout, region, tile_id).join(TERRAIN_HEIGHT_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read terrain height {:?}", path))?;
    let height = serde_json::from_slice(&bytes)?;
    Ok(height)
}
