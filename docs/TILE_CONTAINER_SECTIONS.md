# Tile Container Sections (v1)

All section payloads are little-endian and start with a small internal header.
Unknown tags or versions are skipped safely.

## Canonical tag order

1. META
2. HMAP
3. WMAP
4. LIQD
5. PROP
6. SPLN
7. ADDX

## META (required)

Purpose: tile identity and format versioning.

Header layout:
- version: u16 (v1 = 1)
- reserved: u16
- tile_x: i32
- tile_y: i32
- region_hash: u64
- format_version: u32 (project/world schema version)
- created_timestamp: u64

## HMAP (terrain heightfield)

Header layout:
- version: u16 (v1 = 1)
- reserved: u16
- width: u16
- height: u16
- encoding: u16 (0 = f32)
- reserved: u16
- samples: width * height f32 values

## WMAP (weightmap/splat)

Header layout:
- version: u16 (v1 = 1)
- reserved: u16
- width: u16
- height: u16
- layers: u16
- reserved: u16
- weights: width * height * layers u8 values

## LIQD (liquids)

Header layout:
- version: u16 (v1 = 1)
- reserved: u16
- width: u16
- height: u16
- body_count: u16
- reserved: u16
- mask: width * height u8 values (index into body list)
- bodies: repeated body_count times
  - id: u32
  - height: f32
  - kind: u16 (0 = water, 1 = lava, 2 = slime, 255 = custom)
  - reserved: u16

## PROP (props)

Header layout:
- version: u16 (v1 = 1)
- reserved: u16
- count: u32
- reserved: u32
- records: repeated count times
  - id: u64
  - asset_namespace: u16 length + UTF-8 bytes
  - asset_name: u16 length + UTF-8 bytes
  - translation: 3 * f32
  - rotation: 4 * f32 (quaternion)
  - scale: 3 * f32

Determinism:
- PROP records are sorted by InstanceId ascending before write.

## SPLN (splines)

Reserved for v1. No payload schema yet.

## ADDX (extensions)

Reserved for v1. Tooling-specific payloads must be versioned and documented when used.
