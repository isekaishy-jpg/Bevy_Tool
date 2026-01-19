# CHECKLIST 13 - Reliability (no data loss, corruption handling, recovery)

## Milestone 13.1 - Atomic saves
- [ ] Temp + rename strategy for all writes
- [ ] Prevent partial overwrite

## Milestone 13.2 - Autosave + recovery
- [ ] Autosave snapshots
- [ ] Recovery prompt on crash
- [ ] Backup rotation

## Milestone 13.3 - Corruption handling
- [ ] Validator detects corrupt tiles
- [ ] Quarantine corrupt tiles
- [ ] Project opens with partial data

## Acceptance
- Simulated crash during save does not corrupt the whole project.
