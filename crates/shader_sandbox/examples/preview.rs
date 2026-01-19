use bevy::prelude::*;
use shader_sandbox::preview::{setup_preview_scene, CameraControlPlugin, WaterDebugPlugin};
use shader_sandbox::ShaderSandboxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShaderSandboxPlugin)
        .add_plugins(WaterDebugPlugin)
        .add_plugins(CameraControlPlugin)
        .add_systems(Startup, setup_preview_scene)
        .run();
}
