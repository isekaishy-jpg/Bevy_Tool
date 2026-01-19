use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{ensure_tile_dir, tile_dir, Layout};
use anyhow::Context;
use foundation::ids::{AssetId, InstanceId, TileId};
use serde::{Deserialize, Serialize};
use std::fs;

const PROPS_INSTANCES_FILE: &str = "props.instances.bin";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropsInstances {
    pub format_version: u32,
    pub instances: Vec<PropInstance>,
}

impl PropsInstances {
    pub fn new(instances: Vec<PropInstance>) -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            instances,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropInstance {
    pub id: InstanceId,
    pub asset: AssetId,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

pub fn write_props_instances(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    instances: &PropsInstances,
) -> anyhow::Result<()> {
    let dir = ensure_tile_dir(layout, region, tile_id)?;
    let path = dir.join(PROPS_INSTANCES_FILE);
    let bytes = serde_json::to_vec_pretty(instances)?;
    fs::write(&path, bytes).with_context(|| format!("write props instances {:?}", path))?;
    Ok(())
}

pub fn read_props_instances(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<PropsInstances> {
    let path = tile_dir(layout, region, tile_id).join(PROPS_INSTANCES_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read props instances {:?}", path))?;
    let instances = serde_json::from_slice(&bytes)?;
    Ok(instances)
}
