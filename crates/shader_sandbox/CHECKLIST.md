# Shader Sandbox Checklist (Godot -> Bevy 0.18)

## Goals
- Port the Godot spatial shader into a Bevy 0.18 custom material/pipeline.
- Keep all experimental render graph and screen/depth plumbing isolated to this crate.
- Provide a minimal public API: `ShaderSandboxPlugin` + material types + setup helpers.

## Pipeline Checklist

### 0) Pre-flight
- [ ] Confirm the Godot shader inputs, outputs, and expected visuals.
- [ ] List all required textures: normal1, normal2, height1, height2, edge1, edge2, screen color, depth.
- [ ] Define performance constraints (target FPS, screen resolution, max steps).
- [ ] Decide which features are Phase 1 vs Phase 2 (SSR, refraction, edge).

### 1) Bevy Material Skeleton
- [ ] Create a custom `Material` type with all uniforms mirrored from the Godot shader.
- [ ] Add WGSL shader stub with vertex + fragment entry points.
- [ ] Pipe model/world/view/projection data via `ViewUniform` and `MeshUniform`.
- [ ] Add a `MaterialPlugin` in `ShaderSandboxPlugin`.

### 2) Vertex Displacement
- [ ] Implement UV generation from world position (XZ) + scroll speeds.
- [ ] Sample height textures and displace vertex Y.
- [ ] Validate mesh has tangents; add a note or helper for tangent generation.
- [ ] Verify displacement scale matches Godot.

### 3) Normal Reconstruction
- [ ] Blend normal maps and apply normal strength.
- [ ] Transform normals from tangent space to world/view.
- [ ] Validate lighting response with a simple directional light.

### 4) Fresnel + Base BRDF Controls
- [ ] Implement Schlick fresnel function and verify IOR behavior.
- [ ] Map `ALBEDO`, `ROUGHNESS`, `METALLIC`, `SPECULAR` equivalents.
- [ ] Add debug toggles for quick visual checks.

### 5) Screen Color + Depth Plumbing
- [ ] Add a render graph node (or extraction system) to capture scene color.
- [ ] Expose depth texture from prepass or depth copy.
- [ ] Bind screen color and depth textures into the material.
- [ ] Validate linear depth math vs Bevy projection.

### 6) Refraction
- [ ] Implement refraction UV offsets using normal map and depth.
- [ ] Handle edge cases (out-of-bounds UVs).
- [ ] Match Godot's depth-based volume tinting.
- [ ] Ensure two-sided material (cull disabled) and `front_facing` branches.

### 7) SSR (Screen Space Reflections)
- [ ] Implement ray-march loop in WGSL with `steps` and `far_clip`.
- [ ] Add early-out for out-of-bounds UVs.
- [ ] Blend SSR with fresnel and edge fade.
- [ ] Add quality controls (max steps, resolution scale).

### 8) Edge / Foam / Fade
- [ ] Implement depth-based edge detection.
- [ ] Add foam texture blend or alpha fade based on `foam_or_fade`.
- [ ] Validate against shallow depth transitions.

### 9) Performance + Stability
- [x] Add a debug UI to toggle expensive features (SSR, refraction).
- [x] Clamp or quantize SSR steps for predictable performance.
- [ ] Avoid per-frame allocations in hot paths.

### 10) Integration (Optional)
- [x] Provide an MVP plugin + material builder for integration.
- [ ] Provide a spawn helper for a test mesh + material.
- [x] Keep all helpers behind `shader_sandbox` crate API.
- [ ] Add a feature flag in the app if/when it is integrated.

### 11) Verification
- [ ] Side-by-side compare with Godot output (screenshots or video).
- [ ] Verify behavior under different camera angles and lights.
- [ ] Confirm depth math at near/far ranges.

### 12) Definition of Done
- [ ] Visual parity with Godot for key shots.
- [ ] Document known differences or approximations.
- [ ] Stable in editor at target FPS.
- [ ] All sandbox code stays isolated to this crate.
