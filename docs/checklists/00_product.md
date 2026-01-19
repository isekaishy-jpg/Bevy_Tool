# CHECKLIST 00 - Product definition and acceptance (WoW-Lite)

## Purpose
Lock the product boundaries and quality targets so engineering choices remain aligned to a WoW-style MMO map workflow.

## Milestone 00.1 - Product scope
- [ ] Write `docs/PRODUCT.md` describing:
  - [ ] must-have workflows (terrain, liquids, props, streaming, export, preview)
  - [ ] explicit non-goals for v1
  - [ ] "Definition of Done" for v1

## Milestone 00.2 - Numeric budgets
- [ ] Set and document numeric targets:
  - [ ] world dimensions (tiles)
  - [ ] target loaded tiles (min/typical/max)
  - [ ] frame budget targets (CPU/GPU)
  - [ ] memory target
  - [ ] streaming phase budgets (IO/decode/build/upload)

## Milestone 00.3 - Quality policies
- [ ] Write `docs/QUALITY_BAR.md` covering:
  - [ ] atomic saves
  - [ ] autosave + recovery
  - [ ] corruption quarantine
  - [ ] undo/redo requirements
  - [ ] compatibility and migrations

## Acceptance
- `docs/PRODUCT.md` and `docs/QUALITY_BAR.md` exist and are referenced from README.
