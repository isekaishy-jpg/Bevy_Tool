pub use crate::schema::{WorldSpec, DEFAULT_WORLD_SPEC};

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

pub fn world_spec_from_manifest(manifest: &WorldManifest) -> WorldSpec {
    manifest.world_spec
}

pub fn hash_world_spec_from_manifest(manifest: &WorldManifest) -> u64 {
    hash_world_spec(world_spec_from_manifest(manifest))
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
use crate::schema::WorldManifest;
