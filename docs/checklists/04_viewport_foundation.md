# CHECKLIST 04 — Viewport foundation: camera, input focus, picking, gizmos

## Purpose
Make core interaction reliable.

## Milestone 04.1 — Input focus policy
- [ ] UI consumes input first
- [ ] Viewport captures input only when hovered/focused
- [ ] Escape cancels tool/releases capture

## Milestone 04.2 — RTS camera (editor-grade)
- [ ] Pan + zoom with altitude scaling
- [ ] Optional orbit toggle
- [ ] Frame selection
- [ ] Jump to tile

## Milestone 04.3 — Picking
- [ ] Terrain raycast (heightfield)
- [ ] Prop raycast
- [ ] Hover highlight + selection highlight
- [ ] Selection stored as stable IDs

## Milestone 04.4 — Gizmos and snapping
- [ ] Translate/rotate gizmos
- [ ] Grid/angle snapping
- [ ] Surface snap for placement

## Acceptance
- UI never fights camera.
- Picking remains stable across large coordinate ranges.
