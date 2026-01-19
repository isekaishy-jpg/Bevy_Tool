//! World schema (v0): tiles, layers, and project manifest.

use foundation::ids::TileId;

/// Increment when you introduce breaking changes to the on-disk representation.
pub const WORLD_SCHEMA_VERSION: u32 = 1;

/// Registered logical layers in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Terrain,
    Liquids,
    Objects,
    Metadata,
}

/// Basic project/world manifest. Keep this small and versioned.
#[derive(Debug, Clone)]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub world_name: String,
    pub tile_size_meters: f32,
    pub chunk_resolution: u16,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            schema_version: WORLD_SCHEMA_VERSION,
            world_name: "NewWorld".to_string(),
            tile_size_meters: 533.3333, // Example: WoW-ish tile scale; adjust.
            chunk_resolution: 16,
        }
    }
}

/// Placeholder type representing a tile payload.
#[derive(Debug, Clone)]
pub struct TileRecord {
    pub id: TileId,
    // TODO: layer references / chunk handles
}
