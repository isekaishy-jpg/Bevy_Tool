# Tile Container Format v1 (.tile)

## Summary
Each world tile is stored as a single binary file with:
- A fixed-size header
- A section directory (table of contents)
- A set of payload sections (optionally compressed per-section)

Design goals:
- Streaming-friendly partial reads
- Deterministic output for build pipelines
- Backward/forward compatibility via versioning
- Robustness via checksums and validation

Non-goals (v1):
- Whole-file compression
- Cross-endian portability (little-endian only)
- Cryptographic signing (can be added later)

---

## File naming and layout
Tiles are stored per region:

```
tiles/<region>/
  x####_y####.tile
```

Example:
`tiles/r000/x0010_y0020.tile`

Tile ID embedded in the header **must match** the filename.

---

## Endianness
v1 is **little-endian only**.

---

## Top-level layout

```
[Header (fixed size)]
[Section Directory (N * fixed-size entries)]
[Section Payloads...]
```

Payloads should be aligned to **64 bytes minimum** (4096 optional).

---

## Header (fixed size; 128 bytes recommended)

All fields are little-endian.

Suggested fields:
- `magic: [8]u8` — e.g. `"MTILE\0\0\0"`
- `container_version: u16` — starts at 1
- `endianness: u8` — 1 = little
- `flags: u8` — reserved
- `tile_region: u16`
- `tile_x: i32`
- `tile_y: i32`
- `world_spec_hash: u64` — hash of tile size/resolutions etc.
- `section_count: u32`
- `section_dir_offset: u64`
- `created_unix_ms: u64`
- `reserved: bytes` — pad to fixed size

Header is valid if:
- magic matches
- container_version is supported
- endianness == 1
- section_dir_offset is within file bounds

---

## Section Directory Entry (fixed size; 64 bytes recommended)

Suggested fields:
- `tag: u32` — FourCC (e.g., `HMAP`)
- `section_version: u16`
- `codec: u8` — 0=raw, 1=zstd, 2=lz4
- `flags: u8` — reserved
- `offset: u64` — absolute file offset of payload
- `stored_len: u64` — bytes stored on disk
- `decoded_len: u64` — bytes after decode; equals stored_len if raw
- `crc32: u32` — checksum of stored bytes
- `reserved: bytes` — pad to fixed size

Directory validation rules:
- all offsets and lengths must be within file size
- no overlaps between `[offset, offset+stored_len)`
- (optional) offsets meet the alignment policy

---

## Section decoding
- Whole-file compression is not permitted.
- If compression is used, it is applied **per-section**.
- CRC is computed over the **stored** bytes (post-compression).

---

## Checksums and corruption policy
- `crc32` is computed over the stored payload bytes.
- If a section fails CRC: treat that layer as invalid and continue.
- If header/directory is invalid: tile fails to load; editor continues.

---

## Atomic save policy
Write tile updates via:
1. Write `x####_y####.tile.tmp`
2. Flush (and fsync if desired)
3. Rename `.tmp` → `.tile`

Optional: rotate `.bak` backups.

---

## Determinism policy
- Sections are written in canonical tag order.
- Records inside sections are sorted deterministically.
- Compression settings (if used) must be fixed to ensure stable output.

---

## Compatibility policy
- `container_version` bumps only if header/directory structure changes.
- `section_version` bumps when the internal schema of that section changes.
- Unknown tags/versions are skipped safely.
