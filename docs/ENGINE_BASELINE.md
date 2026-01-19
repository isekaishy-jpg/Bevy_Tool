# Engine Baseline (Bevy 0.18)

## Versions
- Bevy: 0.18.x
- UI: egui via bevy_egui (compatible with Bevy 0.18)

## Feature posture
Default path: Bevy high-level **3D feature profile** with standard renderer.

## Viewport seam
The codebase keeps a boundary between:
- editor tool logic (selection, commands, undo) and
- viewport presentation (camera, picking, rendering)

This is to allow future upgrades:
- improved terrain renderer
- custom wgpu pipeline
- alternative viewport crate

## Camera controllers
Bevy includes basic camera controllers; they may be used for bootstrap/testing.
For the editor, implement an RTS/orbit camera tuned for large world navigation.
