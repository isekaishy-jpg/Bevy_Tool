# CHECKLIST 01 — Repo/workspace and Bevy 0.18 engine baseline

## Purpose
Create a workspace and dependency posture that scales with a long-lived editor.

## Milestone 01.1 — Workspace topology
- [ ] Create crates: foundation, world, runtime, viewport, editor_core, editor_ui, exporter, preview
- [ ] Enforce dependency direction rules (documented)
- [ ] Add feature flags (`dev`, `editor`, `preview`) where appropriate

## Milestone 01.2 — Engine baseline (Bevy 0.18)
- [ ] Pin Bevy 0.18.x
- [ ] Pin bevy_egui compatible with Bevy 0.18.x
- [ ] Document feature profile approach (`features = ["3d"]` as default)
- [ ] Document viewport seam (future custom renderer)

## Milestone 01.3 — CI and hygiene
- [ ] CI: fmt/clippy/test
- [ ] PR template + issue templates
- [ ] Label taxonomy documented

## Acceptance
- `cargo run -p editor` boots, showing a window and an egui frame.
