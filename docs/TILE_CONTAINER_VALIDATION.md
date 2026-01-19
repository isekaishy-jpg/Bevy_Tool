# Tile Container Validation

This document defines the validator contract for `.tile` containers.

## Header validation

- magic must be "TILE"
- container_version must be supported
- endianness must be little
- section_count must be <= 256
- section_dir_offset must be >= header size and within file bounds
- world_spec_hash must match the active world spec (from `world.toml`)

## Directory validation

- stored_len must be > 0
- offset + stored_len must be within file bounds
- payload offsets must not overlap
- payload offsets must not overlap the directory region
- payload alignment must match the container alignment policy
- tags must be ASCII FourCC

## Payload integrity

- crc32 must match the stored bytes
- unknown section tags/versions are skipped safely
- unsupported codecs fail validation for that section

## Section-level schema checks (v1)

- META must be present
- HMAP dimensions must match world spec (if HMAP present)
- WMAP dimensions must match world spec (if WMAP present)
- LIQD dimensions must match world spec (if LIQD present)
- HMAP values must be finite and within the configured range
- LIQD bodies must be finite and mask indices valid
- PROP transforms must be finite

## Quarantine behavior

When quarantine mode is enabled, tiles that fail validation are moved to:

```
worlds/<world_id>/regions/_quarantine/<timestamp>/<region_id>/x####_y####.tile
```

Triggers:
- header read failures (bad magic, invalid directory bounds)
- directory validation failures
- CRC failures or schema validation failures

Quarantine preserves the original region and filename. Users may restore a tile by moving it back
after manual repair.

## Validator outputs

- Human-readable: `ValidationIssue` entries with a message and optional path.
- Machine-readable: JSON array of issues via `validate_project_json(...)`.

## CLI

```
cargo run -p world --bin validate_world -- <project_root>
cargo run -p world --bin validate_world -- --json <project_root>
cargo run -p world --bin validate_world -- --quarantine <project_root>
```
