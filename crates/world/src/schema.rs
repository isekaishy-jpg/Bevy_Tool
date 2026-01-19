//! World schema: tiles, layers, and manifests.

use foundation::ids::TileId;
use serde::{Deserialize, Serialize};

/// Increment when you introduce breaking changes to the project manifest.
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// Increment when you introduce breaking changes to the world manifest.
pub const WORLD_FORMAT_VERSION: u32 = 1;

/// Default world spec used for new worlds and tests.
pub const DEFAULT_WORLD_SPEC: WorldSpec = WorldSpec {
    tile_size_meters: 512.0,
    chunks_per_tile: 16,
    heightfield_samples: 513,
    weightmap_resolution: 256,
    liquids_resolution: 256,
};

/// Registered logical layers in the world.
/// The declaration order is the stable serialization order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerKind {
    Terrain,
    Liquids,
    Props,
    Weightmap,
    Splines,
    Metadata,
}

/// Project manifest stored at the project root.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub project_id: String,
    pub project_name: String,
    pub created_unix_ms: u64,
    pub worlds_dir: String,
    pub assets_dir: String,
    pub exports_dir: String,
    pub cache_dir: String,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            format_version: PROJECT_FORMAT_VERSION,
            project_id: "00000000-0000-0000-0000-000000000000".to_string(),
            project_name: "NewProject".to_string(),
            created_unix_ms: 0,
            worlds_dir: "worlds".to_string(),
            assets_dir: "assets".to_string(),
            exports_dir: "exports".to_string(),
            cache_dir: "cache".to_string(),
        }
    }
}

/// World manifest stored per world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WorldManifest {
    pub format_version: u32,
    pub world_id: String,
    pub world_name: String,
    pub world_spec: WorldSpec,
    pub regions: Vec<RegionManifest>,
}

impl Default for WorldManifest {
    fn default() -> Self {
        Self {
            format_version: WORLD_FORMAT_VERSION,
            world_id: "00000000-0000-0000-0000-000000000000".to_string(),
            world_name: "NewWorld".to_string(),
            world_spec: DEFAULT_WORLD_SPEC,
            regions: Vec::new(),
        }
    }
}

/// Numeric spec that drives tile hashing and validation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WorldSpec {
    pub tile_size_meters: f32,
    pub chunks_per_tile: u16,
    pub heightfield_samples: u16,
    pub weightmap_resolution: u16,
    pub liquids_resolution: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegionManifest {
    pub region_id: String,
    pub name: String,
    pub bounds: RegionBounds,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl RegionBounds {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.min_x <= self.max_x && self.min_y <= self.max_y
    }
}

/// Placeholder type representing a tile payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRecord {
    pub id: TileId,
    // TODO: layer references / chunk handles
}
