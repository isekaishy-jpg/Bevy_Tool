# Checklist 02.5 - Single-file Tile Container Format v1 (.tile)

**Purpose:** Replace JSON/file-per-layer stubs with a streaming-friendly **one-file-per-tile** container that supports partial reads, versioning, validation, and atomic saves.

**Scope:** This checklist covers the **container**, its **sections**, and the **validator contract**. It does not prescribe editor tooling, streaming policies, or export artifacts.

---

## 02.5.1 Decisions and invariants

- [x] Adopt **one file per tile**: `worlds/<world_id>/regions/<region_id>/tiles/x####_y####.tile`
- [x] **Little-endian only** for v1
- [x] **No whole-file compression** (compression is **per section** only)
- [x] Sections must be **independently decodable**
- [x] Unknown section tags/versions must be **safely ignorable**
- [x] `META` section is **required**
- [x] Payload alignment policy documented (recommend **64-byte** minimum; optionally **4096-byte** for page alignment)

**Acceptance criteria**
- `docs/TILE_CONTAINER_FORMAT.md` and `docs/TILE_CONTAINER_SECTIONS.md` exist and are referenced from your world format docs.
- A reviewer can understand the entire on-disk format from the docs alone.

---

## 02.5.2 Container header and directory spec

- [x] Define a **fixed-size header** (recommend **128 bytes**)
  - magic, container_version, endianness, flags
  - tile_id (region/x/y)
  - world_spec_hash
  - section_count + section_dir_offset
  - created timestamp
  - reserved padding
- [x] Define a **fixed-size directory entry** (recommend **64 bytes**)
  - tag (FourCC), section_version, codec, flags
  - offset, stored_len, decoded_len
  - checksum (crc32 of stored bytes)
  - reserved padding
- [x] Define the canonical tag encoding (FourCC packed into u32, ASCII)

**Acceptance criteria**
- A validator can parse header + directory **without** loading any payloads.
- Directory validation rules are explicit: bounds checks, overlap checks, optional alignment checks.

---

## 02.5.3 Section taxonomy and v1 minimum set

- [x] Define canonical tags and required/optional policy:
  - [x] `META` (required)
  - [x] `HMAP` (terrain heightfield; required once terrain editing ships)
  - [x] `WMAP` (weightmap/splat)
  - [x] `LIQD` (liquids)
  - [x] `PROP` (props)
  - [x] `SPLN` (splines)
  - [x] `ADDX` (extension area)
- [x] Define per-section internal header rules (versioned, minimal; dimensions included)
- [x] Define determinism rules:
  - [x] Sections written in canonical tag order
  - [x] Records inside sections sorted deterministically (e.g., `InstanceId` ascending)

**Acceptance criteria**
- Tags and versions are maintained in **one canonical location** (`docs/TILE_CONTAINER_SECTIONS.md`).
- Unknown tags/versions are skipped safely and do not fail the entire tile.

---

## 02.5.4 Atomic save and corruption handling policy

- [x] Implement atomic write strategy:
  - write `*.tile.tmp`, sync, rename to `*.tile`
  - optional: rotate `*.tile.bak`
- [x] Define corruption behavior:
  - if a section CRC fails: report and continue (layer invalid)
  - if directory is invalid: tile fails to load, editor continues
- [x] Establish a validator contract:
  - offset/length bounds checks
  - overlap checks (no two sections overlap)
  - CRC checks (crc32 of stored bytes)

**Acceptance criteria**
- Simulated truncation or bad CRC yields a clear, actionable error and does not crash the editor.

---

## 02.5.5 Implementation deliverables

- [x] `TileContainerReader`:
  - read header
  - read directory
  - `read_section(tag) -> stored_bytes`
  - `decode_section(tag) -> decoded_bytes` (applies codec)
- [x] `TileContainerWriter`:
  - stage sections
  - write header + directory + payloads
  - deterministic output ordering
- [x] Integrate codec support policy (v1 can start with `raw` only; add zstd/lz4 later)
- [x] Unit tests:
  - round-trip (write then read equals original)
  - unknown section skipped safely
  - CRC failure detected
  - overlap detection works
  - bounds detection works

**Acceptance criteria**
- Tests cover the failure modes above.
- Deterministic output is confirmed (same inputs -> identical bytes).

---

## 02.5.6 Validator checklist (add to CI once stable)

**Deliverable:** a headless validator command (library or binary) that can run in CI.

### Required validation rules
- [x] Header validation
  - [x] magic matches
  - [x] supported container_version
  - [x] endianness == little
  - [x] section_count sane (cap; e.g., <= 256)
  - [x] section_dir_offset within file
- [x] Directory validation
  - [x] each entry has non-zero stored_len (unless explicitly allowed)
  - [x] `offset + stored_len` within file bounds
  - [x] no overlaps among stored payload ranges
  - [x] (optional) payload alignment satisfied
  - [x] tags are ASCII FourCC (or explicitly allowed)
- [x] Payload integrity
  - [x] crc32 matches for each payload
  - [x] if crc fails: report section tag + tile id + action taken
- [x] Section-level schema checks (v1)
  - [x] META present
  - [x] If HMAP present: dimensions match world spec (or documented policy)
  - [x] If WMAP present: dimensions match world spec
  - [x] If LIQD present: dimensions match world spec

### Validator outputs
- [x] Human-readable summary (tile id, failing rule, suggested fix)
- [x] Machine-readable output mode (JSON) for tooling pipelines

**Acceptance criteria**
- [x] Validator can scan a directory of tiles and return non-zero exit code on failure.
- [x] Corrupt tiles are reported precisely (tag, offset/len, crc mismatch) without crashing.

---

## Supporting docs

- `docs/TILE_CONTAINER_FORMAT.md`
- `docs/TILE_CONTAINER_SECTIONS.md`
- `docs/TILE_CONTAINER_WORLD_SPEC_HASH.md`
- `docs/TILE_CONTAINER_VALIDATION.md`
