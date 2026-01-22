# CHECKLIST 04 — Viewport System v1 (Bevy 0.18 + egui)

Purpose: Establish a production-grade viewport foundation (layout plumbing, focus gating, camera, coordinate conversions, picking, gizmos, overlays) so CL5+ (terrain/liquids/props/streaming UX) are executable and testable.

Non-goal (v1): render-to-texture / multiple simultaneous viewports. We will preserve a backend seam for it.

---

## Milestone 04.0 — Viewport plumbing (egui layout → camera viewport)

### 04.0.1 Viewport rect resource
- [x] Add `ViewportRect` resource:
  - [x] logical rect (egui points)
  - [x] physical rect (pixels)
  - [x] dpi scale factor
  - [x] “is_valid” flag (viewport exists and non-zero)

### 04.0.2 Derive viewport rect from UI layout
- [x] In the egui layout pass, compute the remaining center rect after panels/docking
- [x] Write to `ViewportRect` every frame
- [x] Clamp within window bounds and avoid negative sizes

### 04.0.3 Apply Bevy camera viewport
- [x] Tag the “editor viewport camera” entity (e.g., `EditorViewportCamera`)
- [x] After UI layout, set `Camera.viewport = Some(Viewport { physical_position, physical_size, .. })`
- [x] Handle resize + dpi changes correctly
- [x] Add a debug overlay toggle to draw the viewport rect outline (optional but useful)

### 04.0.4 Backend seam (future-proof)
- [x] Introduce `ViewportBackend` enum or trait surface:
  - [x] `CameraViewport` (default)
  - [x] `RenderToTexture` (future)
- [x] All tools query viewport geometry through a single `ViewportService` API (no ad hoc math spread around)

**Acceptance**
- 3D rendering is confined to the viewport region; it never draws under egui panels.
- The viewport rect updates correctly when panels are resized/docked/undocked.

---

## Milestone 04.1 — Input focus policy (UI vs viewport)

- [x] UI consumes input first
- [x] Viewport only acts when hovered/focused (cursor inside `ViewportRect`)
- [x] Tool capture semantics:
  - [x] LMB drag captures pointer for the active tool
  - [x] Capture releases on mouse up
- [x] Escape cancels current tool interaction and releases capture
- [x] Keyboard routing:
  - [x] When viewport focused, camera hotkeys work
  - [x] When UI has text focus, viewport hotkeys do not fire

**Acceptance**
- No “UI vs camera fight” under any combination of panel hover, drag, scroll.

---

## Milestone 04.2 — RTS/orbit camera (MMO editor grade)

- [x] Pan (WASD and/or middle-drag)
- [x] Zoom with altitude scaling (speed increases with height)
- [x] Optional orbit (Alt+LMB or similar)
- [x] Frame selection (F)
- [x] Jump to tile coords (Go To…)
- [x] Camera speed presets (slow/normal/fast)
- [x] Safety clamps:
  - [x] min altitude above terrain
  - [x] pitch clamp (avoid flipping)

**Acceptance**
- You can traverse a 255×255-tile region comfortably and precisely.

---

## Milestone 04.3 — Viewport coordinate conversions (foundation for tools)

### 04.3.1 Screen → viewport local coordinates
- [ ] Convert cursor position to viewport-local physical pixels (subtract viewport origin)
- [ ] Reject if outside viewport

### 04.3.2 Viewport local → normalized device coordinates
- [ ] Map viewport-local pixels to NDC ([-1, 1] range)
- [ ] Verify correctness under DPI scaling

### 04.3.3 Viewport local → world ray
- [ ] Implement `viewport_ray(cursor_pos) -> Ray3d` (or equivalent):
  - [ ] uses camera + projection + viewport rect
- [ ] Add a debug “ray hit marker” option (e.g., ray-plane intersection) for verification

**Acceptance**
- Ray origin/direction are stable regardless of panel sizes and DPI.

---

## Milestone 04.4 — Picking and selection

### 04.4.1 Terrain picking (v1)
- [ ] Raycast against:
  - [ ] debug ground plane first (for validation)
  - [ ] heightfield (once terrain exists)
- [ ] Return hit payload:
  - [ ] world position
  - [ ] normal (if available)
  - [ ] tile_id / chunk_id (when terrain model is present)

### 04.4.2 Prop picking (v1)
- [ ] Raycast against prop bounds (AABB/OBB)
- [ ] Prioritize closest hit
- [ ] Hover highlight and selection highlight

### 04.4.3 Stable ID selection model
- [ ] Selection stores stable IDs (tile/chunk/instance ids)
- [ ] Selection remains valid across unload/reload:
  - [ ] If unloaded, show “ghost selection” state
  - [ ] If deleted, selection clears with a log entry

**Acceptance**
- Selection is stable and does not drift when streaming/unloading occurs later.

---

## Milestone 04.5 — Gizmos and snapping

- [ ] Translate gizmo
- [ ] Rotate gizmo
- [ ] Snapping:
  - [ ] Grid snap
  - [ ] Angle snap
  - [ ] Surface snap to terrain (for placement)
- [ ] Gizmo respects focus policy and capture semantics

**Acceptance**
- Gizmo drag is precise and never competes with UI interactions.

---

## Milestone 04.6 — Viewport overlays (must-have for MMO-scale work)

- [ ] Tile boundary overlay
- [ ] Chunk boundary overlay
- [ ] Cursor readout:
  - [ ] world position
  - [ ] tile x/y
  - [ ] chunk x/y
- [ ] Toggle overlay visibility (hotkeys and/or UI)
- [ ] Streaming overlay hook points (no streaming required yet):
  - [ ] “loaded vs unloaded” tile tinting interface stub
  - [ ] “pinned tiles” visualization stub

**Acceptance**
- You can visually confirm which tile/chunk you are editing at all times.

---

## Milestone 04.7 — Viewport diagnostics and regression gates

- [ ] Add a “Viewport Test Scene” mode (dev-only) that:
  - [ ] draws a grid/plane
  - [ ] shows a click marker at ray hit
  - [ ] prints viewport rect and cursor coords
- [ ] Add at least one automated test for rect conversion (logical→physical) if your architecture allows
- [ ] Add one manual regression checklist:
  - [ ] DPI scaling on/off
  - [ ] resizing left/right panels
  - [ ] docking layout changes
  - [ ] window resize

**Acceptance**
- Viewport picking stays correct under layout changes.

---

## Milestone 04.8 — Multi-viewport forward path (design-only, no implementation)

- [ ] Document (short) how we will support:
  - [ ] render-to-texture backend
  - [ ] multiple view panes (perspective/ortho)
  - [ ] thumbnail previews
- [ ] Confirm `ViewportService` API is sufficient for these futures

**Acceptance**
- Switching to RTT later is a backend swap, not a rewrite of tools.

---

## Final Acceptance (CL4)
- Camera and UI never fight; selection survives future streaming unload/reload.
- Viewport renders only in the UI’s viewport region.
- Cursor → ray → hit math remains correct as UI panels move/resize.
- Terrain/prop picking and gizmos are functional enough to begin CL5 terrain work.
