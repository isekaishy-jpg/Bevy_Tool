# CHECKLIST 01 - Repo + workspace + Bevy 0.18 baseline

## Purpose
Create a workspace that will survive a full MMO editor build without collapsing into a monolith.

## Milestone 01.1 - Workspace skeleton
- [x] Create workspace crates (see README): foundation, world, runtime, viewport, editor_core, editor_ui, exporter, preview.
- [x] Enforce dependency direction in docs and code review.
- [x] Add feature flags:
  - [x] `dev` (diagnostics overlays, extra logging)
  - [x] `profiling` (instrumentation)
  - [x] `custom_viewport` (future seam)

## Milestone 01.2 - Bevy 0.18 posture
- [x] Pin Bevy 0.18.x in workspace.
- [x] Pin bevy_egui compatible with Bevy 0.18.
- [x] Document baseline in `docs/ENGINE_BASELINE.md`.

## Milestone 01.3 - CI and templates
- [x] GitHub Actions: fmt/clippy/test on Windows + Linux.
- [x] PR template and issue templates.
- [x] Label taxonomy (docs/GITHUB_LABELS.md).

## Acceptance
- `cargo run -p editor` shows a window and egui frame.
- CI is green.
