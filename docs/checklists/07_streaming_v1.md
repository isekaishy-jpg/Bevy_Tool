# CHECKLIST 07 — Streaming runtime v1: tile/chunk streaming with budgets

## Purpose
Enable MMO-scale editing without loading the whole world.

## Milestone 07.1 — Streaming state machine
- [ ] Tile states: Unloaded/Loading/Loaded/Built/Dirty
- [ ] Cancellation of in-flight loads
- [ ] Dirty rebuild path

## Milestone 07.2 — Budgeted pipeline
- [ ] IO budget per frame
- [ ] CPU decode budget
- [ ] Mesh rebuild budget
- [ ] GPU upload budget

## Milestone 07.3 — Editor controls
- [ ] Pin tile/chunk
- [ ] Force-load radius
- [ ] Priority bias for edited area

## Milestone 07.4 — Observability
- [ ] Streaming stats panel
- [ ] Queue depth and timings
- [ ] Memory reporting per layer

## Acceptance
- Fly across many tiles with stable pacing.
- Terrain and liquids never desynchronize.
