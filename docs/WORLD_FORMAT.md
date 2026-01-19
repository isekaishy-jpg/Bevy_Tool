# World Format (Source)

This is the authoring/source format. Runtime artifacts are produced by the exporter.

## Project layout

```
<project_root>/
  project.toml
  tiles/
    <region>/
      x####_y####.tile
    _quarantine/
      <timestamp>/
        <region>/
          x####_y####.tile
```

## Versioning
- `project.toml` contains `format_version`
- Each `.tile` contains:
  - container version in the header
  - section versions inside each payload
  - world_spec_hash to detect spec mismatches

## Deterministic ordering
- Tiles are ordered by region, then tile coord (x, y)
- Chunks are ordered by chunk coord (x, y)
- Layers use a fixed order: Terrain, Liquids, Props, Weightmap, Splines, Metadata
- Instances are ordered by stable InstanceId (numeric)
- Asset IDs are ordered by namespace, then name
- Bevy Entity IDs are never persisted

## Atomic writes
- Write to `*.tile.tmp`, sync, then rename to `*.tile`
- Existing tiles are rotated to `*.tile.bak` before replace
- Never partially overwrite an existing tile

## Validation
Validator checks:
- container header + directory sanity
- section CRCs and versioned payload decoding
- required sections (META) and dimension checks
- out-of-range values for terrain/liquids/props

See:
- `docs/TILE_CONTAINER_FORMAT.md`
- `docs/TILE_CONTAINER_SECTIONS.md`
- `docs/TILE_CONTAINER_VALIDATION.md`
- `docs/TILE_CONTAINER_WORLD_SPEC_HASH.md`

## Notes
- Legacy per-layer `*.bin`/`*.json` stubs are deprecated in favor of `.tile`.
