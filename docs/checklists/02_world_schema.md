# CHECKLIST 02 — World schema, tiling, layers, persistence, migrations

## Purpose
Define the authoritative world model for MMO-scale map editing.

## Milestone 02.1 — World sizing (adopt defaults)
- [ ] Tile size: 512m
- [ ] Chunks: 16x16 per tile (32m chunks)
- [ ] Heightfield: 513x513 samples
- [ ] Weightmaps: 256x256
- [ ] Liquids mask: 256x256
- [ ] Record in `docs/WORLD_SPEC.md`

## Milestone 02.2 — Stable identifiers
- [ ] TileId, ChunkId
- [ ] Stable InstanceId (props)
- [ ] Namespaced AssetId
- [ ] Layer registry (terrain/liquids/props/weightmaps/splines/metadata)
- [ ] Deterministic serialization ordering rules

## Milestone 02.3 — On-disk layout (source)
- [ ] Project manifest with schema version
- [ ] Tile folder structure and filenames
- [ ] Atomic save strategy

## Milestone 02.4 — Migrations
- [ ] Version fields in all formats
- [ ] Migration pipeline skeleton
- [ ] Tests: migrate old->new for at least one synthetic example

## Milestone 02.5 — Validation and integrity
- [ ] Tile-level validator
- [ ] Corruption quarantine strategy
- [ ] Clear error reporting in editor

## Acceptance
- Create/save/load is lossless.
- Corrupted tile does not crash the editor.
