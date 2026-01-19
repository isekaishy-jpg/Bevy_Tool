# Engine baseline (Bevy 0.18 + egui)

## Target versions

- **Bevy:** 0.18.x
- **UI:** egui via `bevy_egui`

## Feature posture

This repo uses Bevy's high-level feature profile for 3D (`features = ["3d"]`).

### Viewport seam (forward-looking)

The viewport is treated as an architectural seam:

- Default: Bevy renders the viewport.
- Future option: swap to a custom renderer module while keeping the tool framework intact.

To preserve this seam:

- `crates/viewport` must remain UI-agnostic.
- Editor tools operate on stable world data + IDs, not renderer-owned handles.

## Camera

We provide an editor camera in `crates/viewport`. Use built-in Bevy camera controllers only for debugging/bootstrapping; the editor will ultimately maintain its own RTS/orbit camera behavior.
