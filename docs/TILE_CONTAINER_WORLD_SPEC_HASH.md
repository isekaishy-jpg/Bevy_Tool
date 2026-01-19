# world_spec_hash

## Purpose
Tiles must not be mixed across incompatible world specs (tile size, resolutions, encodings).
To prevent subtle corruption, each tile stores a `world_spec_hash` in the container header
(and optionally in META).

## Inputs to the hash (v1)
Include:
- `tile_size_m` (e.g., 512)
- `chunks_per_tile` (e.g., 16)
- `heightfield_resolution` (e.g., 513x513)
- `weightmap_resolution` (e.g., 256x256)
- `liquids_mask_resolution` (e.g., 256x256)
- Section encodings (e.g., HMAP encoding type)

## Policy
- On load: if `tile.world_spec_hash != project.world_spec_hash`, reject the tile with a clear diagnostic.
- On save: always write the project world_spec_hash.

## Implementation note
Any stable 64-bit hash is acceptable (e.g., xxhash64). The hash function and inputs must be documented.
