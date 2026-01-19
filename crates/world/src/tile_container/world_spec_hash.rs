#[derive(Debug, Clone, Copy)]
pub struct WorldSpec {
    pub tile_size_meters: f32,
    pub chunks_per_tile: u16,
    pub heightfield_samples: u16,
    pub weightmap_resolution: u16,
    pub liquids_resolution: u16,
}

pub const DEFAULT_WORLD_SPEC: WorldSpec = WorldSpec {
    tile_size_meters: 512.0,
    chunks_per_tile: 16,
    heightfield_samples: 513,
    weightmap_resolution: 256,
    liquids_resolution: 256,
};

pub fn hash_world_spec(spec: WorldSpec) -> u64 {
    let data = format!(
        "tile_size_meters={};chunks_per_tile={};heightfield_samples={};weightmap_resolution={};liquids_resolution={}",
        spec.tile_size_meters,
        spec.chunks_per_tile,
        spec.heightfield_samples,
        spec.weightmap_resolution,
        spec.liquids_resolution
    );
    fnv1a_64(data.as_bytes())
}

pub fn hash_region(region: &str) -> u64 {
    fnv1a_64(region.as_bytes())
}

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
