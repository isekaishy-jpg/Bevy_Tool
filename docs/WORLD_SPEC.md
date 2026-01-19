# World spec (WoW-style defaults)

These values are selected to match a World of Warcraft–style world authoring model: heightfield terrain, splat-painted materials, liquids as a layer, and tile streaming.

## Partitioning

- **Tile size:** 512m x 512m
- **Chunks per tile:** 16 x 16
- **Chunk size:** 32m x 32m

## Terrain

- **Heightfield samples per tile:** 513 x 513
  - 512 intervals across the tile -> ~1m sample spacing
  - +1 sample avoids seam duplication issues

## Terrain materials

- **Weightmap resolution:** 256 x 256 per tile (~2m texel)
- **Initial material layers:** 4 (expandable)

## Liquids

- **Coverage mask resolution:** 256 x 256 per tile
- **Height model (v1):** per contiguous liquid body (one height scalar per body)
- **Types:** enum/material id (water/lava/slime/etc.)

## Props/doodads

- Instances stored per tile with stable IDs
- Basic LOD tiers: 3

## Streaming (editor)

- Primary unit: tile
- Secondary unit: chunk (rebuild/LOD)

## Notes

These defaults are not immutable, but they are used as the reference assumptions throughout `docs/checklists/`.
