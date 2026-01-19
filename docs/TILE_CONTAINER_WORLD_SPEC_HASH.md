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

## Defaults

The current defaults match `docs/WORLD_SPEC.md`:
- tile_size_meters = 512.0
- chunks_per_tile = 16
- heightfield_samples = 513
- weightmap_resolution = 256
- liquids_resolution = 256

## Region hash

The header also stores `region_hash` as FNV-1a 64-bit of the region name string.
