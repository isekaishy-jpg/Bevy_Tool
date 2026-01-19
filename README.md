# MMO World Editor (Bevy 0.18 + egui)

A multi-tool repository focused first on a World of Warcraft–style MMO world editor:

- Large tiled open worlds (tile streaming)
- Heightfield terrain sculpting
- Texture/weightmap painting
- Liquids authoring (lakes/rivers/oceans)
- Props/doodads placement
- Deterministic export pipeline + artifact-only runtime preview

## Engine baseline

- Bevy: **0.18.x**
- UI: **egui via bevy_egui**

See `docs/ENGINE_BASELINE.md`.

## Quick start

```bash
# From repository root
cargo run -p editor
```

## Repository layout

- `apps/editor`: Editor entrypoint
- `crates/world`: World schema, persistence, validation, migrations
- `crates/runtime`: Tile/chunk streaming runtime
- `crates/viewport`: Camera, picking, gizmos (UI-agnostic)
- `crates/editor_core`: Tools framework, undo/redo, selection model
- `crates/editor_ui`: egui docking and panels
- `crates/exporter`: Artifact build pipeline
- `crates/preview`: Artifact-only preview mode

## Documentation

- `docs/checklists/`: Extensive checklists toward a finished product
- `docs/WORLD_SPEC.md`: Concrete WoW-style world sizing defaults
- `docs/WORLD_FORMAT.md`: On-disk formats (source + artifacts)
- `docs/ROADMAP.md`: High-level milestone overview
