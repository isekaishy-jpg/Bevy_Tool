# World format

This document describes the on-disk layout for **source** (editable) data and **artifacts** (runtime-consumable) data.

## Source data layout (editable)

At project root:

- `project.toml` (or `project.json`) — versioned project manifest
- `tiles/` — tile data grouped by region

Suggested tile layout:

- `tiles/<region>/<x>_<y>/tile.meta`
- `tiles/<region>/<x>_<y>/terrain.height`
- `tiles/<region>/<x>_<y>/terrain.weightmap`
- `tiles/<region>/<x>_<y>/liquids.mask`
- `tiles/<region>/<x>_<y>/liquids.meta`
- `tiles/<region>/<x>_<y>/props.instances`
- `tiles/<region>/<x>_<y>/splines.bin`
- `tiles/<region>/<x>_<y>/metadata.json`

### Atomic saves

All writes must be atomic at the tile-file level:

1. write to temp file in same directory
2. fsync as appropriate
3. rename to target

## Artifact layout (runtime)

At project root:

- `artifacts/<build_id>/` — versioned export output

Suggested artifacts per tile:

- `terrain/` — chunk meshes and/or heightfield+LOD recipes
- `liquids/` — chunk surfaces, masks, metadata
- `props/` — instance lists and streaming groups
- `materials/` — baked splat/weight textures
- `splines/` — baked road/river meshes or spline data
- `nav/` — placeholder for future navigation exports

## Versioning and migrations

- Every file format includes a version header.
- Project manifest includes a schema version.
- Migrations are explicit transforms from old -> new.
