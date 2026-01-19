# Tile Container Validation Contract

This document defines the minimum validation checks and required error reporting for `.tile` files.

## Validator modes
- **Scan mode:** validate every tile under a directory; report all failures.
- **Single-tile mode:** validate one tile; print precise diagnostics.
- **Machine-readable output:** optional JSON output for CI/pipelines.

## Required checks

### 1) Header
- magic matches expected value
- container_version supported
- endianness == little
- section_count is within a sane cap (recommend <= 256)
- section_dir_offset within file bounds
- tile_id matches filename (region/x/y)

### 2) Directory
For each directory entry:
- tag is valid FourCC (or explicitly allowed)
- offset and stored_len within file bounds
- stored_len > 0 unless explicitly allowed
- decoded_len >= 0 (and equals stored_len for raw)

Global directory checks:
- no overlaps between payload ranges
- optional: payload offsets satisfy alignment policy

### 3) Payload integrity
- crc32 of stored bytes matches directory entry

Policy on failure:
- Section CRC failure invalidates that layer; tile can still be partially loaded.
- Header or directory failure invalidates the entire tile.

### 4) Section-level schema checks (v1)
- META present
- If HMAP present: width/height and encoding valid
- If WMAP present: width/height and layer_count valid
- If LIQD present: width/height and encodings valid

## Error reporting requirements
Each error must include:
- tile_id (region/x/y)
- failing rule name
- relevant tag (if section-related)
- offset/len (if applicable)
- suggested action (e.g., "re-export tile" or "delete corrupt tile")

## Exit codes
- 0: all tiles valid
- 1: one or more tiles invalid
- 2: validator internal error
