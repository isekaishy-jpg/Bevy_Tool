# Checklist 02.5 — Single-file Tile Container Format v1 (.tile)

**Purpose:** Replace JSON/file-per-layer stubs with a streaming-friendly **one-file-per-tile** container that supports partial reads, versioning, validation, and atomic saves.

**Scope:** This checklist covers the **container**, its **sections**, and the **validator contract**. It does not prescribe editor tooling, streaming policies, or export artifacts.

---

## 02.5.1 Decisions and invariants

- [ ] Adopt **one file per tile**: `tiles/<region>/x####_y####.tile`
- [ ] **Little-endian only** for v1
- [ ] **No whole-file compression** (compression is **per section** only)
- [ ] Sections must be **independently decodable**
- [ ] Unknown section tags/versions must be **safely ignorable**
- [ ] `META` section is **required**
- [ ] Payload alignment policy documented (recommend **64-byte** minimum; optionally **4096-byte** for page alignment)

**Acceptance criteria**
- `docs/TILE_CONTAINER_FORMAT.md` and `docs/TILE_CONTAINER_SECTIONS.md` exist and are referenced from your world format docs.
- A reviewer can understand the entire on-disk format from the docs alone.

---

## 02.5.2 Container header and directory spec

- [ ] Define a **fixed-size header** (recommend **128 bytes**)
  - magic, container_version, endianness, flags
  - tile_id (region/x/y)
  - world_spec_hash
  - section_count + section_dir_offset
  - created timestamp
  - reserved padding
- [ ] Define a **fixed-size directory entry** (recommend **64 bytes**)
  - tag (FourCC), section_version, codec, flags
  - offset, stored_len, decoded_len
  - checksum (crc32 of stored bytes)
  - reserved padding
- [ ] Define the canonical tag encoding (FourCC packed into u32, ASCII)

**Acceptance criteria**
- A validator can parse header + directory **without** loading any payloads.
- Directory validation rules are explicit: bounds checks, overlap checks, optional alignment checks.

---

## 02.5.3 Section taxonomy and v1 minimum set

- [ ] Define canonical tags and required/optional policy:
  - [ ] `META` (required)
  - [ ] `HMAP` (terrain heightfield; required once terrain editing ships)
  - [ ] `WMAP` (weightmap/splat)
  - [ ] `LIQD` (liquids)
  - [ ] `PROP` (props)
  - [ ] `SPLN` (splines)
  - [ ] `ADDX` (extension area)
- [ ] Define per-section internal header rules (versioned, minimal; dimensions included)
- [ ] Define determinism rules:
  - [ ] Sections written in canonical tag order
  - [ ] Records inside sections sorted deterministically (e.g., `InstanceId` ascending)

**Acceptance criteria**
- Tags and versions are maintained in **one canonical location** (`docs/TILE_CONTAINER_SECTIONS.md`).
- Unknown tags/versions are skipped safely and do not fail the entire tile.

---

## 02.5.4 Atomic save and corruption handling policy

- [ ] Implement atomic write strategy:
  - write `*.tile.tmp` → flush/sync policy (documented) → rename to `*.tile`
  - optional: rotate `*.tile.bak`
- [ ] Define corruption behavior:
  - if a section CRC fails: mark that layer invalid, keep project open
  - if directory is invalid: tile fails to load, editor continues
- [ ] Establish a validator contract:
  - offset/length bounds checks
  - overlap checks (no two sections overlap)
  - CRC checks (crc32 of stored bytes)

**Acceptance criteria**
- Simulated truncation or bad CRC yields a clear, actionable error and does not crash the editor.

---

## 02.5.5 Implementation deliverables

- [ ] `TileContainerReader`:
  - read header
  - read directory
  - `read_section(tag) -> stored_bytes`
  - `decode_section(tag) -> decoded_bytes` (applies codec)
- [ ] `TileContainerWriter`:
  - stage sections
  - write header + directory + payloads
  - deterministic output ordering
- [ ] Integrate codec support policy (v1 can start with `raw` only; add zstd/lz4 later)
- [ ] Unit tests:
  - round-trip (write then read equals original)
  - unknown section skipped safely
  - CRC failure detected
  - overlap detection works
  - bounds detection works

**Acceptance criteria**
- Tests cover the failure modes above.
- Deterministic output is confirmed (same inputs → identical bytes).

---

## 02.5.6 Validator checklist (add to CI once stable)

**Deliverable:** a headless validator command (library or binary) that can run in CI.

### Required validation rules
- [ ] Header validation
  - [ ] magic matches
  - [ ] supported container_version
  - [ ] endianness == little
  - [ ] section_count sane (cap; e.g., <= 256)
  - [ ] section_dir_offset within file
- [ ] Directory validation
  - [ ] each entry has non-zero stored_len (unless explicitly allowed)
  - [ ] `offset + stored_len` within file bounds
  - [ ] no overlaps among stored payload ranges
  - [ ] (optional) payload alignment satisfied
  - [ ] tags are ASCII FourCC (or explicitly allowed)
- [ ] Payload integrity
  - [ ] crc32 matches for each payload
  - [ ] if crc fails: report section tag + tile id + action taken
- [ ] Section-level schema checks (v1)
  - [ ] META present
  - [ ] If HMAP present: dimensions match project spec (or documented policy)
  - [ ] If WMAP present: dimensions match project spec
  - [ ] If LIQD present: dimensions match project spec

### Validator outputs
- [ ] Human-readable summary (tile id, failing rule, suggested fix)
- [ ] Machine-readable output mode (JSON) for tooling pipelines

**Acceptance criteria**
- Validator can scan a directory of tiles and return non-zero exit code on failure.
- Corrupt tiles are reported precisely (tag, offset/len, crc mismatch) without crashing.

---

## Supporting docs

- `docs/TILE_CONTAINER_FORMAT.md`
- `docs/TILE_CONTAINER_SECTIONS.md`
- `docs/TILE_CONTAINER_WORLD_SPEC_HASH.md`
- `docs/TILE_CONTAINER_VALIDATION.md`
