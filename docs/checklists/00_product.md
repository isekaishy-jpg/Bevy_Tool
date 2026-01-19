# CHECKLIST 00 - Product definition and acceptance (WoW-Lite)

## Purpose
Lock the product boundaries and quality targets so engineering choices remain aligned to a WoW-style MMO map workflow.

## Milestone 00.1 - Product scope
- [x] Write `docs/PRODUCT.md` describing:
  - [x] must-have workflows (terrain, liquids, props, streaming, export, preview)
  - [x] explicit non-goals for v1
  - [x] "Definition of Done" for v1

## Milestone 00.2 - Numeric budgets
- [x] Set and document numeric targets:
  - [x] world dimensions (tiles)
    - Region size (v1 default): 256 x 256 tiles (~131 km per side at 512m tiles)
    - Supported world: multiple regions; stream region-by-region (not all mounted at once)
    - Format ceiling: 1024 x 1024 tiles per region
    - Stretch target: 512 x 512 tiles (scale/format test)
  - [x] target loaded tiles (min/typical/max)
    - Min: 3 x 3 (9 tiles)
    - Typical: 5 x 5 (25 tiles)
    - Max: 9 x 9 (81 tiles)
  - [x] frame budget targets (CPU/GPU)
    - Target: 60 FPS (16.7 ms), CPU 10 ms, GPU 10 ms
    - Acceptable floor: 30 FPS (33.3 ms), CPU 20 ms, GPU 20 ms
  - [x] memory target
    - RAM: min 3 GB, typical 6 GB, max 10-12 GB
    - VRAM: min 2 GB, typical 4 GB, max 8 GB
  - [x] streaming phase budgets (IO/decode/build/upload)
    - IO scheduling: <= 2 ms/frame; 2 tiles in-flight; assume >= 200 MB/s SSD, >= 50 MB/s HDD
    - Decode: <= 2 ms/frame average; cap background decode tasks to <= 50% CPU cores
    - Build: <= 4 ms/frame average; caps: terrain 2 chunks/frame, liquids 4 chunks/frame, props batching 1 tile/frame
    - Upload: <= 2 ms/frame average; <= 8 MB/frame
    - High radius mode (9 x 9): allow 3-4 tiles in-flight; terrain up to 3-4 chunks/frame if still responsive

## Milestone 00.3 - Quality policies
- [x] Write `docs/QUALITY_BAR.md` covering:
  - [x] atomic saves
  - [x] autosave + recovery
  - [x] corruption quarantine
  - [x] undo/redo requirements
  - [x] compatibility and migrations

## Acceptance
- [x] `docs/PRODUCT.md` and `docs/QUALITY_BAR.md` exist and are referenced from README.
