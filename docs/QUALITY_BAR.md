# Quality bar

## Data safety

- Tile writes are atomic.
- Autosave produces recoverable snapshots.
- Corrupted tiles are quarantined; project opens with partial content.

## Determinism

- Stable IDs in files.
- Canonical ordering in serialized output.
- Export output is deterministic given identical inputs.

## Performance

- No long UI stalls during common editing (budgeted rebuilds and streaming).
- Streaming and rebuild queues are observable in-editor.
