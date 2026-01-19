# CHECKLIST 06 - Liquids v1 (lakes/ocean + river hooks)

## Milestone 06.1 - Data model
- [ ] Coverage mask 256x256 per tile
- [ ] Contiguous body metadata: height scalar + type
- [ ] Serialization + migration + validation

## Milestone 06.2 - Rendering
- [ ] Chunked water surface generation
- [ ] Z-fighting avoidance policy documented
- [ ] Debug overlays (mask, body ids)

## Milestone 06.3 - Editing tools
- [ ] Paint/erase coverage
- [ ] Set height (numeric + pick-from-terrain)
- [ ] (Optional) fill region

## Milestone 06.4 - Undo/redo + budgets
- [ ] Patch-based deltas
- [ ] Bounded rebuild work

## Acceptance
- Liquids save/load/stream correctly; tools feel immediate.
