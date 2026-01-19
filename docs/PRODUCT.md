# Product Definition (WoW-Lite MMO Map Editor)

## Target quality
A **World of Warcraft (2004) style** world editor workflow and runtime output:
- Large tiled world
- Heightfield terrain
- Painted terrain textures
- Liquids (lakes/rivers/ocean)
- Doodads/props
- Streaming

Visual target is "WoW-like" in complexity and readability, but may use modern rendering conveniences.

## v1 workflows (must-have)
1. Create/open project
2. Navigate a large world (RTS camera)
3. Sculpt terrain (raise/lower/smooth/flatten)
4. Paint surface materials (weightmaps/splat)
5. Author liquids (lakes/rivers/oceans) (paint + set height + type)
6. Place props/doodads and edit transforms
7. Stream tiles while editing (with budgets and pinning)
8. Export runtime artifacts
9. Artifact-only preview mode

## Non-goals (v1)
- Collaborative multi-user editing (multiplayer)
- Modeling suite (in-editor)
- Full scripting/quest IDE
- Full gameplay simulation

## Definition of Done (v1)
- Stable save formats with migrations
- Autosave + crash recovery
- Validator tool (headless) + export tool (headless)
- Performance budgets met on representative content
- Packaging for target OS (at least Windows)

## Quality bar (v1)
- Atomic tile saves
- Autosave + crash recovery
- Corruption quarantine
- Deterministic export

## Numeric budgets (v1)
- Target FPS and acceptable spikes
- Memory target
- Streaming budgets (IO/CPU/GPU)
