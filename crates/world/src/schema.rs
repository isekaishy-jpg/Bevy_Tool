//! World schema (v0): tiles, layers, and project manifest.

use foundation::ids::TileId;
use serde::{Deserialize, Serialize};

/// Increment when you introduce breaking changes to the on-disk representation.
pub const WORLD_SCHEMA_VERSION: u32 = 1;

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

/// Basic project/world manifest. Keep this small and versioned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub world_name: String,
    pub tile_size_meters: f32,
    pub chunk_resolution: u16,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            format_version: WORLD_SCHEMA_VERSION,
            world_name: "NewWorld".to_string(),
            tile_size_meters: 512.0,
            chunk_resolution: 16,
        }
    }
}

/// Placeholder type representing a tile payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRecord {
    pub id: TileId,
    // TODO: layer references / chunk handles
}
