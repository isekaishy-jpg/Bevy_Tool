# CHECKLIST 05 — Terrain v1: data, render, sculpt, undo/redo, LOD hooks

## Purpose
Implement WoW-style terrain editing.

## Milestone 05.1 — Terrain data
- [ ] Heightfield (513x513) per tile
- [ ] Chunk mapping (16x16)
- [ ] Dirty tracking (chunk-level)
- [ ] Seam rules documented

## Milestone 05.2 — Terrain rendering
- [ ] Chunk mesh generation
- [ ] Seam stitching across chunks/tiles
- [ ] Async rebuild with budgets
- [ ] Debug overlays: chunk boundaries, normals

## Milestone 05.3 — Sculpt tools
- [ ] Raise/lower
- [ ] Smooth
- [ ] Flatten-to-height
- [ ] Brush UI: radius/strength/falloff + cursor

## Milestone 05.4 — Undo/redo
- [ ] Patch-based deltas
- [ ] Stroke grouping

## Milestone 05.5 — Derived data hooks
- [ ] Cached normals
- [ ] Collision proxy placeholder
- [ ] LOD placeholder strategy

## Acceptance
- Sculpt across tile boundaries seamlessly.
- Undo/redo exact.
- No long stalls on typical strokes.
