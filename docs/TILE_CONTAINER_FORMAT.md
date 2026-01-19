# Tile Container Format (.tile)

This file describes the on-disk container used by the editor to store a single tile per file.
All values are little-endian.

## File layout

```
| 128B header | N * 64B directory entries | section payloads (aligned) |
```

### Header (128 bytes)

Fields (in order):
- magic: 4 bytes, ASCII "TILE"
- container_version: u16
- endianness: u16 (1 = little)
- flags: u32 (reserved)
- tile_x: i32
- tile_y: i32
- region_hash: u64 (FNV-1a)
- world_spec_hash: u64 (FNV-1a)
- section_count: u32
- section_dir_offset: u64 (byte offset to directory)
- created_timestamp: u64 (unix seconds)
- reserved: padding to 128 bytes

### Directory entry (64 bytes)

Fields (in order):
- tag: u32 (FourCC, ASCII)
- section_version: u16
- codec: u16 (0 = raw)
- flags: u32 (reserved)
- offset: u64 (payload offset)
- stored_len: u64 (stored bytes length)
- decoded_len: u64 (decoded bytes length)
- crc32: u32 (of stored bytes)
- reserved: padding to 64 bytes

## Alignment

Payloads are aligned to 64 bytes. Alignment padding bytes are undefined and ignored.

## Tag encoding

Tags are ASCII FourCC encoded into a u32. Unknown tags are skipped safely.

## Atomic writes

Tiles are written to `*.tile.tmp`, synced, and then renamed to `*.tile`.
If an existing tile is present, it is rotated to `*.tile.bak` before replacement.

## Validation rules (summary)

- header magic, version, endianness
- directory bounds and non-overlap
- CRC32 for payloads
- required section policy (META required)

See `docs/TILE_CONTAINER_VALIDATION.md` for the complete validator contract.
