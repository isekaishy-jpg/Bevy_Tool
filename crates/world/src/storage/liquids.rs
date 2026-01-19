use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{ensure_tile_dir, tile_dir, Layout};
use anyhow::Context;
use foundation::ids::TileId;
use serde::{Deserialize, Serialize};
use std::fs;

const LIQUIDS_MASK_FILE: &str = "liquids.mask.bin";
const LIQUIDS_META_FILE: &str = "liquids.meta.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiquidsMask {
    pub format_version: u32,
    pub size: u16,
    pub mask: Vec<u8>,
}

impl LiquidsMask {
    pub fn new(size: u16, mask: Vec<u8>) -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            size,
            mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiquidsMeta {
    pub format_version: u32,
    pub bodies: Vec<LiquidBody>,
}

impl LiquidsMeta {
    pub fn new(bodies: Vec<LiquidBody>) -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            bodies,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiquidBody {
    pub id: u32,
    pub height: f32,
    pub kind: LiquidKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LiquidKind {
    Water,
    Lava,
    Slime,
    Custom(String),
}

pub fn write_liquids_mask(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    mask: &LiquidsMask,
) -> anyhow::Result<()> {
    let dir = ensure_tile_dir(layout, region, tile_id)?;
    let path = dir.join(LIQUIDS_MASK_FILE);
    let bytes = serde_json::to_vec_pretty(mask)?;
    fs::write(&path, bytes).with_context(|| format!("write liquids mask {:?}", path))?;
    Ok(())
}

pub fn read_liquids_mask(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<LiquidsMask> {
    let path = tile_dir(layout, region, tile_id).join(LIQUIDS_MASK_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read liquids mask {:?}", path))?;
    let mask = serde_json::from_slice(&bytes)?;
    Ok(mask)
}

pub fn write_liquids_meta(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    meta: &LiquidsMeta,
) -> anyhow::Result<()> {
    let dir = ensure_tile_dir(layout, region, tile_id)?;
    let path = dir.join(LIQUIDS_META_FILE);
    let bytes = serde_json::to_vec_pretty(meta)?;
    fs::write(&path, bytes).with_context(|| format!("write liquids meta {:?}", path))?;
    Ok(())
}

pub fn read_liquids_meta(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<LiquidsMeta> {
    let path = tile_dir(layout, region, tile_id).join(LIQUIDS_META_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read liquids meta {:?}", path))?;
    let meta = serde_json::from_slice(&bytes)?;
    Ok(meta)
}
