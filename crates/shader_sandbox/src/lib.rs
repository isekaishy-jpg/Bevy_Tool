//! Shader sandbox for experimental rendering work.

use bevy::asset::embedded_asset;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

use crate::material::WaterMaterial;

pub struct ShaderSandboxPlugin;

impl Plugin for ShaderSandboxPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(
            app,
            "crates/shader_sandbox/src",
            "shaders/water_material.wgsl"
        );
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
    }
}

pub mod material;
pub mod mvp;
pub mod pipeline;
pub mod preview;
pub mod render_graph;
pub use mvp::WaterMvpPlugin;
