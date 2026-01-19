# Bevy MMO World Editor (Starter Scaffold)

This repository is a starter scaffold for a Bevy + egui 3D world editor targeting large, MMO-style open worlds (WoW-like), with tile/chunk streaming, terrain sculpting, and liquids.

## Status
Early scaffold: compiles once you add Bevy dependencies and start implementing the app assembly.

## Workspace layout
See `docs/ARCHITECTURE.md`.

## Getting started
1. Install Rust (stable).
2. Add Bevy + egui dependencies in `apps/editor/Cargo.toml`.
3. Implement Bevy App assembly in `crates/editor_core` and UI panels in `crates/editor_ui`.

## Roadmap
See `docs/ROADMAP.md`.

## License
MIT (see `LICENSE`).
