# CHECKLIST 04 - Viewport foundation (camera, picking, gizmos)

## Milestone 04.1 - Input focus policy
- [ ] UI consumes input first
- [ ] Viewport only acts when hovered/focused
- [ ] Escape cancels/release capture

## Milestone 04.2 - RTS/orbit camera
- [ ] Pan/zoom with altitude scaling
- [ ] Optional orbit
- [ ] Frame selection
- [ ] Jump to tile coords

## Milestone 04.3 - Picking and selection
- [ ] Terrain raycast (heightfield)
- [ ] Prop raycast
- [ ] Hover and selection highlight
- [ ] Selection uses stable IDs

## Milestone 04.4 - Gizmos and snapping
- [ ] Translate + rotate
- [ ] Grid snap + angle snap
- [ ] Surface snap for placement

## Acceptance
- Camera and UI never fight; selection survives streaming unload/reload.
