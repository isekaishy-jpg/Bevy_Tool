# Issue Pack (Milestones M0–M10)

This is the canonical issue plan. Create GitHub milestones M0–M10 and file epics under each.

## M0 — Repo + Workspace + CI
**Epic:** Repo Baseline
- Initialize Rust workspace and crate layout
- Add GitHub Actions CI (fmt/clippy/test)
- Add contributor docs and templates

## M1 — World Schema v0 (Tiles/Chunks/Layers/IDs)
**Epic:** World Model
- Define TileId/ChunkId and coordinate conventions
- Define layer registry (terrain, liquids, objects, metadata)
- Project manifest format + versioning policy
- On-disk layout (folders per tile, chunk blobs, metadata)

## M2 — Editor Shell + Egui Docking + Input Focus
**Epic:** Editor Frame
- Egui docking layout (Viewport, Outliner, Inspector, Console)
- Input routing policy (UI first, viewport capture)
- Command registration + hotkey map

## M3 — Viewport MVP (Camera, Picking, Gizmos)
**Epic:** Viewport Core
- RTS/orbit camera with altitude-scaled speed
- Picking: raycast selection + hover highlight
- Gizmos: translate/rotate/scale + grid

## M4 — Terrain Display v0 (Heightfield + Chunk Mesh)
**Epic:** Terrain Rendering
- Heightfield storage per tile/chunk
- Chunk mesh generation (async) + seam stitching
- Terrain debug material + lighting baseline

## M5 — Terrain Sculpting v0 (Raise/Lower + Undo/Redo)
**Epic:** Terrain Tools
- Brush model + cursor visualization
- Raise/Lower tool implementation
- Undo/Redo command stack for terrain strokes
- Performance guardrails for brush strokes

## M6 — Liquids v0 (Water Layer + Basic Editing)
**Epic:** Liquids Layer
- Liquids data model (coverage mask + height + type)
- Liquids rendering (simple planes + sorting)
- Liquids tools: paint mask + set height
- Shoreline interaction v0 (anti-z-fight + rules)

## M7 — Tile Streaming Runtime v0 (Editor-Grade)
**Epic:** Streaming
- Streaming manager (active set, priority, budgeted IO)
- Editor overrides: pin tiles, force load radius
- Cache policy and memory caps
- Streaming supports terrain + liquids

## M8 — Object Placement + World Composition
**Epic:** Props
- Prefab/archetype model + stable asset IDs
- Placement tool (raycast to terrain, snap)
- Outliner by tile/layer + selection sync

## M9 — Texturing + Roads/Rivers (Biomes and Splines)
**Epic:** Surface Authoring
- Weightmap painting (splat layers)
- Spline editing (roads/rivers)
- River integration with liquids (spline bakes liquids)

## M10 — Export Pipeline + Runtime Preview
**Epic:** Export
- Exporter: tile/chunk artifact build
- Validation (missing assets, corrupt tiles)
- Runtime preview mode (artifact-only load)
