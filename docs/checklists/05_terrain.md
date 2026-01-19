# CHECKLIST 05 - Terrain v1 (render + sculpt + undo + LOD hooks)

## Milestone 05.1 - Terrain data model
- [ ] Heightfield 513x513 stored per tile
- [ ] Chunk dirty tracking 16x16
- [ ] Seam rules defined

## Milestone 05.2 - Rendering
- [ ] Per-chunk mesh generation
- [ ] Stitch across chunks/tiles
- [ ] Async rebuild with budgets
- [ ] Debug overlays (chunk boundaries, normals)

## Milestone 05.3 - Sculpt tools
- [ ] Raise/lower
- [ ] Smooth
- [ ] Flatten-to-height
- [ ] Brush cursor + hotkeys for size/strength

## Milestone 05.4 - Undo/redo
- [ ] Patch-based deltas
- [ ] Stroke grouping

## Milestone 05.5 - Forward hooks
- [ ] LOD placeholder (distance-based)
- [ ] Collision artifact placeholder

## Acceptance
- Edits across tile boundaries are seamless; undo/redo exact; rebuild latency bounded.
