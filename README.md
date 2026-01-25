# MMO World Editor (Bevy 0.18 + egui)

An in-progress world editor for large, tiled MMO-style worlds. The long-term goal is a full authoring suite (terrain, liquids, props, streaming, export), with a focused viewport foundation first.

This repo is under active development. Expect breaking changes while the checklists evolve.

## Engine baseline

- Bevy: **0.18.x**
- UI: **egui via bevy_egui**

See `docs/ENGINE_BASELINE.md`.

## Quick start

```bash
# From repository root
cargo run -p editor
```

## Current focus

- Viewport foundation (camera, input routing, picking groundwork, debug overlays)
- World/region manifests and stable IDs
- Streaming and authoring systems staged behind checklists

See `docs/checklists/` for the source of truth on progress and requirements.

## Repository layout

- `apps/editor`: Editor entrypoint
- `crates/world`: World schema, persistence, validation, migrations
- `crates/runtime`: Tile/chunk streaming runtime
- `crates/viewport`: Camera, picking, gizmos (UI-agnostic)
- `crates/editor_core`: Tools framework, undo/redo, selection model
- `crates/editor_ui`: egui docking and panels
- `crates/exporter`: Artifact build pipeline
- `crates/preview`: Artifact-only preview mode

## Development workflow

- Format: `cargo fmt`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Test: `cargo test`

Manual regression steps for the viewport live in `docs/checklists/04_viewport.md`.

## Documentation

- `docs/checklists/`: Extensive checklists toward a finished product
- `docs/PRODUCT.md`: Product scope and definition of done
- `docs/QUALITY_BAR.md`: Quality policies and acceptance criteria
- `docs/WORLD_SPEC.md`: Concrete WoW-style world sizing defaults
- `docs/WORLD_FORMAT.md`: On-disk formats (source + artifacts)
- `docs/ROADMAP.md`: High-level milestone overview
