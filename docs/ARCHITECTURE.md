# Architecture

## High-level

- `world`: schema + persistence + validation (source of truth)
- `runtime`: streaming + budgets + lifecycle (consumes world data)
- `viewport`: camera/picking/gizmos (UI-agnostic)
- `editor_core`: tool framework + undo/redo + selection
- `editor_ui`: egui docking + panels (consumes editor_core state)
- `exporter`: converts source data into runtime artifacts
- `preview`: loads artifacts only

## Key invariants

- Editor UI does not own world data; it presents and edits it through tools.
- Selection is represented with stable IDs (not Bevy entity handles).
- Streaming may unload tiles at any time; tools must tolerate that.
- Viewport is a seam: rendering can change without rewriting tools.
