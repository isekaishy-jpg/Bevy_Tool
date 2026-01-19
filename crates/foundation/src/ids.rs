//! Stable identifiers for tiles/chunks/etc.

/// World tile coordinate (x, y) in a 2D grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

/// A stable tile identifier. Extend with zone/layer later if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileId {
    pub coord: TileCoord,
}

/// Chunk coordinate within a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkCoord {
    pub x: u16,
    pub y: u16,
}

/// A stable chunk identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkId {
    pub tile: TileId,
    pub coord: ChunkCoord,
}
