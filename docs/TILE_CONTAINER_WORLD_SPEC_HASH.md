# Tile Container World Spec Hash

Each `.tile` header stores `world_spec_hash` to detect mismatches between a tile and the active
project/world spec. The hash is not a security feature; it is a quick consistency check.

## Hash algorithm

- Algorithm: FNV-1a 64-bit
- Input: ASCII string with stable key ordering

Input string format:

```
tile_size_meters=<f32>;
chunks_per_tile=<u16>;
heightfield_samples=<u16>;
weightmap_resolution=<u16>;
liquids_resolution=<u16>
```

## Manifest fields

The world spec hash is derived from `project.toml` fields:
- tile_size_meters
- chunk_resolution (chunks per tile)
- heightfield_resolution
- weightmap_resolution
- liquids_resolution

Defaults match `docs/WORLD_SPEC.md` when those fields are not overridden.

## Region hash

The header also stores `region_hash` as FNV-1a 64-bit of the region name string.
