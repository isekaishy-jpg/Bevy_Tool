# World Spec (WoW-Lite defaults)

These defaults are chosen to match a WoW-era workflow while remaining scalable.

## Partitioning
- Tile size: **512m x 512m**
- Chunks per tile: **16 x 16**
- Chunk size: **32m x 32m**

## Terrain
- Heightfield resolution per tile: **513 x 513** samples
  - 512 meters / 512 intervals = ~1m spacing
  - The +1 row/col enables seam-consistent sampling
- Mesh generation: per chunk derived from the tile heightfield

## Materials
- Weightmap resolution per tile: **256 x 256**
- Start with 4 layers per tile; allow expansion later

## Liquids
- Coverage mask: **256 x 256** per tile
- Water height: v1 stores a scalar per contiguous body + mask
  - later: per-cell heights and flow maps

## Props
- Instances stored per tile
- 3 LOD tiers (simple), distances configurable

## Streaming
- Primary unit: tile
- Secondary unit: chunk (rebuild/LOD)
- Budgeted pipeline: IO -> decode -> build -> GPU upload
