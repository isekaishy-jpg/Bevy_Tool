# CHECKLIST 02 - World schema v1 (tiles/chunks/layers/IDs/migrations)

## Purpose
Define the authoritative world representation for a WoW-style MMO map editor.

## Milestone 02.1 - Adopt and document world spec
- [ ] Write `docs/WORLD_SPEC.md` (or confirm the defaults):
  - [ ] tile 512m
  - [ ] 16x16 chunks (32m)
  - [ ] heightfield 513x513
  - [ ] weightmap 256x256
  - [ ] liquids mask 256x256

## Milestone 02.2 - Stable IDs
- [ ] TileId, ChunkId, LayerId, InstanceId, AssetId defined in `crates/world`.
- [ ] Deterministic ordering rules documented.
- [ ] No Bevy Entity IDs persisted.

## Milestone 02.3 - Layer registry
- [ ] Terrain layer
- [ ] Liquids layer
- [ ] Props layer
- [ ] Weightmap/material layer
- [ ] Spline layer
- [ ] Metadata layer

## Milestone 02.4 - Persistence layout
- [ ] Define and document `docs/WORLD_FORMAT.md`.
- [ ] Implement load/save stubs for:
  - [ ] project manifest
  - [ ] tile meta
  - [ ] terrain height
  - [ ] liquids mask/meta
  - [ ] props instances

## Milestone 02.5 - Migrations + validation
- [ ] Format versioning.
- [ ] Migration pipeline exists (even if no migrations yet).
- [ ] Validator can scan project and report errors.
- [ ] Corrupt tile quarantine policy implemented.

## Acceptance
- Create -> save -> reload is lossless for stubs.
- Corrupt tile does not crash the editor.
