# CHECKLIST 04 — Viewport System v1 (Single Source of Truth)

This checklist consolidates all viewport work into one file. Every item below is **in scope** for v1.
If an item is not listed here, it is **out of scope**.

---

## Global Definition of Done

CHECKLIST 04 is **done** when all milestones (04.0–04.7) meet their respective DoD and:

- [ ] All behaviors are stable under:
  - [ ] window resize
  - [ ] viewport panel resize
  - [ ] docking/layout changes
  - [ ] DPI / scale factor changes
- [ ] Input routing is deterministic (no “double-consume” between UI and viewport).
- [ ] All debug/overlay features are gated behind a master toggle and do not impose measurable cost when disabled.
- [ ] A dev regression scene exists and is referenced from this checklist (see 04.7).
- [ ] Docs:
  - [ ] This checklist reflects the actual implementation (no “paper requirements”).
  - [ ] Any new public types/resources/commands are documented where they live.

---

## 04.0 Viewport plumbing (egui layout → camera viewport)

**Goal:** A stable, UI-driven viewport rectangle that correctly configures the Bevy camera viewport.

### Requirements
- [x] Maintain a `ViewportRect` resource with:
  - [x] logical origin/size (egui points)
  - [x] physical origin/size (pixels)
  - [x] `scale_factor`
  - [x] `is_valid`
- [x] Derive `ViewportRect` from UI layout every frame.
- [x] Apply the rect to the Bevy camera viewport correctly (including resize/DPI changes).
- [x] Provide a UI-agnostic `ViewportService` boundary:
  - [x] editor UI writes rect
  - [x] viewport crate consumes rect
  - [x] other systems read from `ViewportService` (not UI internals)

### DoD
- [x] Moving/resizing/docking the viewport panel never breaks camera viewport placement.
- [x] `ViewportRect.is_valid` is false whenever the viewport is non-renderable (zero size, not visible, etc.).
- [x] A one-line on-screen debug readout (dev-only is fine) shows rect + scale factor for verification.

---

## 04.1 Input focus policy (UI vs viewport)

**Goal:** Deterministic routing between UI widgets and viewport interactions.

### Requirements
- [x] UI consumes input first.
- [x] Viewport only acts when hovered/focused.
- [x] Capture semantics:
  - [x] mouse drag operations capture until release
  - [x] Escape cancels/clears capture (and cancels current tool interaction if any)
- [x] Keyboard routing:
  - [x] viewport hotkeys only when viewport focused (or when explicitly allowed by design)

### DoD
- [x] There is no state where both UI and viewport respond to the same click/drag.
- [x] Escape reliably exits any capture mode and returns to a neutral state.
- [x] A small dev-only HUD line indicates current focus/capture state (UI-focused, viewport-hovered, viewport-captured).

---

## 04.2 Camera (RTS/orbit, MMO editor grade)

**Goal:** A practical editor camera that supports large-scale world authoring.

### Requirements
- [x] Pan/orbit/zoom with speed scaling.
- [x] Frame selection / focus point.
- [x] Go To Tile navigation.
- [x] Safety clamps (e.g., min height, max pitch).
- [x] Clear “camera mode” vs “tool mode” behavior.

### DoD
- [x] Camera controls remain correct under viewport rect changes (04.0).
- [x] Go To Tile:
  - [x] clamps or errors clearly when the request is out of bounds (no silent failure)
  - [x] never produces NaNs / invalid transforms
- [x] A small, fixed set of manual regression steps is documented in 04.7 and passes consistently.

Notes
- Go To Tile clamps against the active region bounds, warns inline + log when clamped, and errors if no active region is selected.
- Camera input is disabled while a tool has capture to avoid conflicts.

---

## 04.3 Viewport coordinate conversions (foundation for tools)

**Goal:** Correct, testable conversions for all future picking and tool placement.

### Requirements
- [x] Convert:
  - [x] screen → viewport-local pixels
  - [x] viewport-local → NDC
  - [x] viewport-local → world ray (camera origin + direction)
- [x] Verification path:
  - [x] cast ray against a flat plane and render a hit marker (temporary, dev-only allowed)
- [ ] Robustness:
  - [x] works under layout changes (docking, resizing)
  - [x] works under DPI changes

### DoD
- [x] The hit marker lands where expected at multiple camera angles and zoom levels.
- [x] Leaving/entering the viewport (hover changes) does not produce spurious rays or stale hits.

Notes
- The tile grid overlay (04.5) represents region scope; the old dev ground grid has been removed.

---

## 04.4 World cursor, picking, and selection model

**Goal:** One authoritative “world cursor” output and consistent selection semantics.

### Requirements

#### 04.4.1 World cursor service output
- [x] Produce a single authoritative `WorldCursor` payload when the mouse is inside the viewport:
  - [x] `has_hit: bool`
  - [x] `hit_pos_world: Vec3`
  - [x] `hit_normal_world: Vec3` (or fallback normal)
  - [x] `region_id/name` (placeholder acceptable until regions are real)
  - [x] `tile_x/tile_y`
  - [x] `chunk_x/chunk_y` (when chunking exists)
  - [x] `snap_pos_world` (equals hit until snapping exists)
  - [x] `snap_kind` (off/tile/chunk/subgrid)

#### 04.4.2 Terrain picking v1
- [x] Raycast against debug plane initially.
- [x] Closest hit wins.

#### 04.4.3 Prop picking v1
- [x] Raycast against prop bounds (AABB/OBB acceptable for v1).
- [x] Distinguish hover vs selection.

#### 04.4.4 Stable ID selection model
- [x] Selection stores stable IDs (tile id now; entity ids later).
- [x] Handle missing/unloaded targets gracefully (clear with explicit feedback).

### DoD
- [x] The “world cursor reticle” appears only when `has_hit` is true and the mouse is inside the viewport.
- [x] Hover never implicitly selects; selection is explicit.
- [x] Selection state survives camera motion and viewport resize without drift.

---

## 04.5 Spatial context overlays (tile/chunk/sub-grid/bounds) + streaming hooks

**Goal:** Make space legible and debuggable without rendering the full world.

### Requirements

#### 04.5.0 Overlay framework + toggles
- [x] Overlay toggles (panel/menu) for:
  - [x] cursor readout
  - [x] fps readout
  - [x] present mode selector (vsync/auto no vsync/immediate)
  - [x] tile grid
  - [x] chunk grid
  - [x] sub-grid
  - [x] region bounds
  - [x] hover highlight
  - [x] selection highlight
  - [x] debug markers (ray hit, viewport rect)
  - [x] streaming visualization hooks (loaded/pending/dirty/pinned/error)
- [x] Hotkeys:
  - [x] master overlays on/off
  - [x] cycle snap level (coarse-fine)
  - [x] cycle sub-grid spacing
- [x] Persist overlay settings in editor prefs.

Notes
- Overlay Options opens a standalone window so combo boxes (present mode, sub-grid spacing) work without closing the menu.

#### 04.5.1 Cursor readout
- [x] Only show when:
  - [x] mouse is inside viewport AND
  - [x] `WorldCursor.has_hit` is true
- [x] Readout includes:
  - [x] world pos
  - [x] region id/name
  - [x] tile coords
  - [x] chunk coords (when available)
  - [x] active tool
  - [x] snap mode + spacing

#### 04.5.2 Tile grid overlay
- [x] Render scope shows the full region bounds (fixed grid).
- [x] Highlight hovered tile.
- [x] Alignment exactly matches tile coordinate system.

#### 04.5.3 Chunk grid overlay
- [x] Same scoping rule as tile grid (local only).
- [x] Highlight hovered chunk when enabled.

Notes
- Chunk grid uses a fixed 4x4 tile window centered on the camera tile.

#### 04.5.4 Sub-grid overlay
- [x] Spacing levels: 32, 16, 8, 4, 2, 1 (default 8).
- [x] Anchor to a stable origin. Choose and document exactly one:
  - [ ] world origin, OR
  - [x] region origin
- [x] Render scope is local (use the same scoping rule as tile/chunk).
- [x] LOD rules:
  - [x] far: hidden
  - [x] mid: chunk only
  - [x] near: chunk + sub-grid
  - [x] very near: allow 1m

Notes
- Sub-grid uses the same fixed 4x4 tile window as chunk grid, anchored to region origin for alignment.

#### 04.5.5 Region bounds overlay + enforcement
- [x] Render region bounds.
- [x] Warn near edge/outside.
- [x] Enforce: tools cannot author outside bounds (block with clear message).

#### 04.5.6 Hover + selection visualization
- [x] Hover highlight:
  - [x] tile
  - [x] chunk (when enabled)
- [x] Selection highlight:
  - [x] selected tile v1
  - [x] reserved path for entities later
- [x] Clear selection affordance:
  - [x] Esc clears selection
  - [x] click empty clears selection

#### 04.5.7 Streaming visualization hooks
- [x] Tile state model (fed by streaming later):
  - [x] loaded
  - [x] pending_load
  - [x] dirty
  - [x] pinned
  - [x] error / quarantined
- [x] Visualization conventions:
  - [x] define precedence rules (e.g., error overrides dirty)
  - [x] outline vs fill rules
- [x] Provide an API surface for the streaming system to feed these states later.

Notes
- Streaming overlays use tile outlines; error tiles add a diagonal cross.
- Grid overlays visualize tile/chunk/sub-grid extents only; they do not imply geometry is loaded or rendered.

#### 04.5.8 Overlay performance gates
- [ ] Overlays never iterate full region (e.g., never scan 255x255).
- [x] Debug counters:
  - [x] lines drawn
  - [x] tiles considered per frame
- [ ] Enabling overlays does not cause large frame drops.

### DoD
- [x] All overlays can be toggled independently and via a master toggle.
- [x] Overlay settings persist across restarts.
- [x] With overlays disabled, overlay systems do near-zero work (no large loops, no allocations).
- [x] With overlays enabled, scope limits are respected and counters confirm bounded work.

---

## 04.6 Snap system + gizmos

**Goal:** Centralize snapping so gizmos and tools share one contract.

### Requirements

#### 04.6.1 Snap system
- [ ] Snap modes:
  - [ ] off
  - [ ] tile snap
  - [ ] chunk snap
  - [ ] sub-grid snap (uses active sub-grid spacing)
- [ ] Controls:
  - [ ] cycle snap mode (coarse↔fine)
  - [ ] cycle sub-grid spacing
  - [ ] toggle snap on/off
- [ ] Snap applies to:
  - [ ] gizmo translation
  - [ ] placement/move (future consumers)
  - [ ] brush center alignment (future consumers)

#### 04.6.2 Gizmos
- [ ] Translate gizmo.
- [ ] Rotate gizmo.
- [ ] Gizmo respects focus/capture semantics.
- [ ] Gizmo respects snap system where applicable.

### DoD
- [ ] Snapping behavior is consistent across:
  - [ ] world cursor snapped position
  - [ ] gizmo translation
- [ ] Snap mode and spacing are visible in the cursor readout.
- [ ] Changing snap settings never causes selection/tool state corruption.

---

## 04.7 Diagnostics and regression gates (viewport + overlays)

**Goal:** Make viewport correctness non-negotiable and easy to verify.

### Requirements
- [ ] Dev-only “Viewport Regression Scene”:
  - [ ] flat plane with known scale
  - [ ] tile/chunk/sub-grid overlays + toggles
  - [ ] cursor readout stable
  - [ ] debug marker for ray hit point
  - [ ] viewport rect debug marker
  - [ ] panel resizing/docking does not break picking/overlays
- [ ] Manual regression checklist is written here and kept current.

### Manual regression checklist
- [x] Resize the viewport panel in both directions; verify ray hit marker stays correct.
- [x] Change docking/layout; verify viewport rect + hit marker stays correct.
- [x] Toggle DPI/scale factor (or simulate); verify viewport rect and ray hit remain correct.
- [x] Toggle overlays master on/off; verify bounded work counters.
- [x] Enter/exit capture modes; verify Escape always returns to neutral.
- [ ] Go To Tile: out-of-bounds requests clamp to active region bounds and emit a visible warning (no NaNs).
- [ ] Go To Tile: no active region selected shows a clear error and does not move the camera.
- [ ] Tool capture active: camera hotkeys and mouse camera controls are disabled until capture ends.
- [x] Ray hit marker: verify placement at multiple camera angles and zoom levels.
- [x] Ray hit marker: leaving/entering the viewport does not create stale hits.
- [x] Prop debug cube: toggle on shows wireframe marker at (2, 0.5, 2).
- [x] Prop debug cube: hover/click selects prop and logs selection set.

### DoD
- [ ] The regression scene exists, is easy to launch, and is referenced from developer docs/README.
- [ ] The manual regression checklist above passes reliably.
