# World Format (Source)

This is the authoring/source format. Runtime artifacts are produced by the exporter.

## Project layout

```
<project_root>/
  project.toml
  tiles/
    <region>/
      <x>_<y>/
        tile.meta.json
        terrain.height.bin
        terrain.weightmap.bin
        liquids.mask.bin
        liquids.meta.json
        props.instances.bin
        splines.bin
```

## Versioning
- `project.toml` contains `format_version`
- Each per-tile file is assumed to match the project format version

## Deterministic ordering
- Tiles are ordered by region, then tile coord (x, y)
- Chunks are ordered by chunk coord (x, y)
- Layers use a fixed order: Terrain, Liquids, Props, Weightmap, Splines, Metadata
- Instances are ordered by stable InstanceId (numeric)
- Asset IDs are ordered by namespace, then name
- Bevy Entity IDs are never persisted

## Atomic writes
- Write to `*.tmp` then rename
- Never partially overwrite an existing file

## Validation
Validator checks:
- missing files
- version mismatch
- corrupt headers
- out-of-range values

## Notes
- Early stubs may store JSON payloads in `*.bin` files until binary formats are defined.
- Binary formats should start with a small header:
  - magic
  - version
  - dimensions
  - checksum (optional)
