# Tile Container Sections (Tags and v1 Schemas)

This document defines canonical tags and v1 internal schemas. Each section begins
with a small internal header that includes its own version and dimensions.

All numeric values are little-endian.

---

## Canonical tags
- `META` — tile metadata (required)
- `HMAP` — terrain heightfield
- `WMAP` — terrain weightmap/splat
- `LIQD` — liquids
- `PROP` — prop instances
- `SPLN` — splines
- `ADDX` — extension (experimental)

FourCC encoding:
Use ASCII tags packed into a u32.

---

## META v1 (required)

Purpose:
- Quick summary used by streaming and editor UI without loading heavy payloads

Suggested contents (example; keep small):
- `world_spec_hash: u64`
- `layer_presence_bits: u64`
- `min_height_m: f32`
- `max_height_m: f32`
- `edit_revision: u32` (optional)
- reserved/padding

Recommended size: < 512 bytes.

---

## HMAP v1 (heightfield)

Internal header:
- `version: u16` (=1)
- `width: u16` (513)
- `height: u16` (513)
- `encoding: u8` (0=u16_scaled, 1=i16_scaled, 2=f32_raw)
- `reserved: u8`
- `base_height_m: f32` (used if scaled)
- `step_m: f32` (used if scaled)

Payload:
- Samples `[width*height]` in chosen encoding, row-major

v1 recommendation:
- `u16_scaled` for size and speed.

---

## WMAP v1 (weightmap/splat)

Internal header:
- `version: u16` (=1)
- `width: u16` (256)
- `height: u16` (256)
- `layer_count: u8` (4 in v1)
- `reserved: u8`

Payload (recommended v1):
- RGBA weights: `[width*height*4] u8`
  - channels correspond to material slots 0..3

---

## LIQD v1 (liquids)

Goal:
Support WoW-like lakes/oceans with brush editing and later river baking.

Internal header:
- `version: u16` (=1)
- `width: u16` (256)
- `height: u16` (256)
- `mask_encoding: u8` (0=u8 per cell, 1=bitset)
- `body_id_encoding: u8` (0=none, 1=u8, 2=u16)

Body table:
- `body_count: u32`
- each body:
  - `body_id: u32`
  - `liquid_type: u16` (enum/material id)
  - `reserved: u16`
  - `height_m: f32`

Payload grids:
- Coverage mask grid
- Optional body-id grid referencing body table

v1 recommendation:
- u8 coverage + u8 body_id for simplicity.

---

## PROP v1 (prop instances)

Internal header:
- `version: u16` (=1)
- `instance_count: u32`

Instance record (fixed-size, deterministic order by `instance_id`):
- `instance_id: u64`
- `asset_id: u64`
- `pos: [3]f32`
- `rot: [4]f32` (quat)
- `scale: [3]f32`
- `flags: u32`
- `variant: u32`

Future options:
- Quantized transforms
- Palettes for assets/material variants

---

## SPLN v1 (splines)

Internal header:
- `version: u16` (=1)
- `spline_count: u32`

Per spline:
- `spline_id: u64`
- `spline_type: u16` (road/river/etc)
- `reserved: u16`
- `point_count: u32`
- `width_m: f32`
- optional: metadata blob

Points:
- positions in tile-local coordinates
- optional tangents

---

## ADDX v1 (extension)

A generic container for experimental data.
Policy:
- Best-effort load
- Safe to ignore if unknown
