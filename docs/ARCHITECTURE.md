# Architecture

## Core principles
1. **Stable IDs**: all persistent references use stable IDs (not Bevy Entity IDs).
2. **Source vs Artifacts**: the editor authors source; exporter builds runtime artifacts.
3. **Budgeted streaming**: IO/decode/build/upload are time-sliced.
4. **Viewport seam**: viewport can evolve independently.

## Crate responsibilities
- `foundation`: shared primitives (ids, errors, small utilities)
- `world`: schema, persistence, validation, migrations
- `runtime`: streaming manager + tile/chunk lifecycle
- `viewport`: camera/picking/gizmos + integration points
- `editor_core`: tools framework, command stack, selection model, project state/prefs
- `editor_ui`: egui UI, docking, panels
- `exporter`: artifact build pipeline
- `preview`: artifact-only preview runtime

## Dependency direction
Keep dependencies flowing toward lower-level crates to avoid cycles:
- `foundation`: no project crate dependencies
- `world` -> `foundation`
- `runtime` -> `world`, `foundation`
- `viewport` -> `foundation`
- `editor_core` -> `world`, `runtime`, `foundation`
- `editor_ui` -> `editor_core`, `viewport`
- `exporter` -> `world`, `runtime`, `foundation`
- `preview` -> `runtime`, `viewport`
- `apps/editor` wires everything together

Code review enforces these directions; exceptions must be documented.

## Plugin composition
- `apps/editor` constructs Bevy `App`
- `editor_ui` registers UI systems
- `viewport` registers camera + picking
- `runtime` registers streaming

## Commands and undo/redo
All user operations should be expressed as commands:
- command has `apply()` and `revert()`
- command records deltas (patch region)
- command is the unit of undo/redo

## Data flow
- UI emits command intents
- editor_core validates and produces commands
- world layer is modified
- runtime/viewport observe and rebuild derived state

## Project model
- Global prefs: `editor.toml` (recents, last project)
- Project root: `project.toml` + `.editor/` (editor-only state)
- Worlds: `worlds/<world_id>/world.toml` (world spec + region registry)
- Regions: `regions/<region_id>/tiles/x####_y####.tile` with defined bounds
- Active selection: editor tracks a current world id and an active region id (runtime `ActiveRegion`) to scope viewport commands like Go To Tile and bounds clamping.
