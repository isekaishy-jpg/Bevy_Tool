# CHECKLIST 06 — Liquids v1: lakes/oceans + river hooks

## Purpose
Add liquids early to avoid schema/streaming/export rework.

## Milestone 06.1 — Data model
- [ ] Coverage mask (256x256)
- [ ] Liquid bodies metadata (height scalar, type)
- [ ] Serialization + validation + migration fields

## Milestone 06.2 — Rendering
- [ ] Chunked water surface generation
- [ ] Z-fighting avoidance policy
- [ ] Debug overlays (mask/body IDs)

## Milestone 06.3 — Editing tools
- [ ] Paint/erase coverage
- [ ] Set height (numeric + pick-from-terrain)
- [ ] Optional fill region

## Milestone 06.4 — Undo/redo + performance
- [ ] Patch-based deltas
- [ ] Budgeted rebuilds

## Acceptance
- Lakes persist across save/load.
- Liquids stream coherently with terrain.
