<INSTRUCTIONS>
## Project Guardrails
- Bevy is pinned to `0.18` in workspace dependencies. Do not upgrade without an explicit request.
- Favor small, focused modules and plugins. Avoid mega-modules, mega-resources, and "god" plugins.
- If a file grows beyond ~300 lines or 3+ responsibilities, split it.
- If a system or resource grows beyond 5-7 fields of unrelated concerns, split it by domain.
- Prefer data-oriented, event-driven flows (components + events + systems) over monolithic managers.
- Keep UI panels decoupled; each panel should be its own module with minimal shared state.

## Context Bloat Avoidance
- Avoid central "everything" structs. Use multiple resources scoped by subsystem (editor, world, viewport, runtime).
- Avoid large enums or match trees in a single file; move per-domain logic into separate modules.
- Keep top-level crates thin; push details into `mod` files to keep compile and mental load low.
- Prefer registries (panel registry, tool registry, command registry) over huge switch statements.
- When adding new features, update `docs/ARCHITECTURE.md` and/or `docs/ROADMAP.md` if structure changes.

## Checklists
### New Feature Checklist
- Define the smallest owning crate/module.
- Define inputs/outputs (events/resources/components).
- Add only the dependencies you need; prefer workspace deps.
- Add tests if behavior is non-trivial.
- Update docs if architecture or flow changed.

### UI / Editor Panels Checklist
- Add a new panel enum entry and tab implementation (keep panel files small).
- Do not hard-code layout changes without a reset or migration path.
- Avoid panel-to-panel direct coupling; use shared resources or events.
- Keep panel UI logic separate from data mutations (command/event for actions).

### World Data / Storage Checklist
- Prefer stable serialization with versioned schemas.
- Avoid loading the entire world into memory when chunking will suffice.
- Keep storage formats decoupled from editor-only types.
- Add migration notes when schema changes.

### Runtime / Streaming Checklist
- Keep streaming systems bounded; avoid scanning all entities each frame.
- Prefer spatial indexing or chunk iterators.
- Guard against unbounded caches; add LRU or size limits.
- Collect perf metrics early to catch regressions.

### Testing / Validation Checklist
- If adding core logic, add at least one unit test.
- For editor UX changes, add a smoke test path (manual steps ok).
- Ensure debug output/log spam is avoided in hot loops.

## Long-Term Risks to Track
- Schema migrations and backwards compatibility for world data.
- Undo/redo memory growth (delta compaction, snapshotting).
- Chunk streaming performance and stutter under large worlds.
- Editor UI performance with many panels/tabs.
- Plugin order dependencies and hidden resource coupling.
- Asset pipeline compatibility (formats, compression, mesh/terrain scaling).
- Multi-user or networked editing sync complexity.
- Save/load time and format stability across Bevy versions.
</INSTRUCTIONS>
