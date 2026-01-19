# CHECKLIST 07 - Streaming runtime v1 (tile streaming + budgets + metrics)

## Milestone 07.1 - Tile/chunk state machine
- [ ] Unloaded -> Loading -> Loaded(Data) -> Built(Renderable)
- [ ] Cancellation of in-flight loads
- [ ] Dirty rebuild path

## Milestone 07.2 - Budgeted pipeline
- [ ] IO budget
- [ ] CPU decode budget
- [ ] Mesh rebuild budget
- [ ] GPU upload budget

## Milestone 07.3 - Editor controls
- [ ] Pin tile/chunk
- [ ] Force-load radius
- [ ] Priority bias to edited area

## Milestone 07.4 - Observability
- [ ] Streaming stats panel (queues, timings, memory)
- [ ] Debug overlays (tile bounds)

## Acceptance
- Traverse large map without stalls; layers stay coherent.
