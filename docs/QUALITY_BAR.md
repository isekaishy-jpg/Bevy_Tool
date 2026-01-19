# Quality Bar

## Checklist baseline (v1)
- Atomic tile saves
- Autosave + crash recovery
- Corruption quarantine
- Deterministic export

## Data safety
- All on-disk writes are atomic (temp + rename).
- Autosave is enabled by default (configurable).
- On crash, user is prompted to recover.

## Undo/redo
- Any edit that changes world data must be undoable.
- Undo records deltas (patches), not full-tile copies.

## Corruption handling
- A single corrupt tile must not prevent opening the project.
- Corrupt tiles are quarantined with an actionable error report.

## Performance
- Editor remains interactive under streaming churn.
- Rebuild work is budgeted; no unbounded frame stalls.

## Compatibility
- Save format is versioned.
- Migrations exist and are tested.
