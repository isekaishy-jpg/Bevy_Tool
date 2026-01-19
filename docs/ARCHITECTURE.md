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
- `editor_core`: tools framework, command stack, selection model
- `editor_ui`: egui UI, docking, panels
- `exporter`: artifact build pipeline
- `preview`: artifact-only preview runtime

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
