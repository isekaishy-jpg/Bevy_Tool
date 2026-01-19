# CHECKLIST 13 — Reliability: autosave, crash recovery, corruption handling

## Purpose
Professional tool behavior with no-data-loss expectations.

## Milestone 13.1 — Save safety
- [ ] Atomic tile saves
- [ ] Autosave snapshots
- [ ] Backup rotation

## Milestone 13.2 — Corruption handling
- [ ] Detect corrupt tiles
- [ ] Quarantine + continue opening
- [ ] Diagnostics bundle

## Milestone 13.3 — Recovery UX
- [ ] Recovery prompt
- [ ] Restore autosave

## Acceptance
- Simulated crash during save does not corrupt the full project.
