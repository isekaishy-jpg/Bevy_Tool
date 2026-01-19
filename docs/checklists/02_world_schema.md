# CHECKLIST 02 - World schema v1 (tiles/chunks/layers/IDs/migrations)

## Purpose
Define the authoritative world representation for a WoW-style MMO map editor.

## Milestone 02.1 - Adopt and document world spec
- [x] Write `docs/WORLD_SPEC.md` (or confirm the defaults):
  - [x] tile 512m
  - [x] 16x16 chunks (32m)
  - [x] heightfield 513x513
  - [x] weightmap 256x256
  - [x] liquids mask 256x256

## Milestone 02.2 - Stable IDs
- [x] TileId, ChunkId, LayerId, InstanceId, AssetId defined in `crates/world`.
- [x] Deterministic ordering rules documented.
- [x] No Bevy Entity IDs persisted.

## Milestone 02.3 - Layer registry
- [x] Terrain layer
- [x] Liquids layer
- [x] Props layer
- [x] Weightmap/material layer
- [x] Spline layer
- [x] Metadata layer

## Milestone 02.4 - Persistence layout
- [x] Define and document `docs/WORLD_FORMAT.md`.
- [x] Implement load/save stubs for:
  - [x] project manifest
  - [x] tile meta
  - [x] terrain height
  - [x] liquids mask/meta
  - [x] props instances

## Milestone 02.5 - Migrations + validation
- [x] Format versioning.
- [x] Migration pipeline exists (even if no migrations yet).
- [x] Validator can scan project and report errors.
- [x] Corrupt tile quarantine policy implemented.

## Acceptance
- [x] Create -> save -> reload is lossless for stubs.
- [x] Corrupt tile does not crash the editor.
