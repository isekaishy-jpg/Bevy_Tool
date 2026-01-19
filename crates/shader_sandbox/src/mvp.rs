//! MVP-facing plugin and helpers for wiring the water material into other apps.

use bevy::asset::embedded_asset;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

use crate::material::{WaterMaterial, WaterMaterialParams};

const WATER_SHADER_PATH: &str = "crates/shader_sandbox/src";
const WATER_SHADER_FILE: &str = "shaders/water_material.wgsl";

/// Plugin that registers the MVP water material and its embedded shader.
pub struct WaterMvpPlugin;

impl Plugin for WaterMvpPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, WATER_SHADER_PATH, WATER_SHADER_FILE);
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
    }
}

/// Texture handles required by the MVP water material.
#[derive(Debug, Clone)]
pub struct WaterMaterialTextures {
    pub normal1: Handle<Image>,
    pub normal2: Handle<Image>,
    pub height1: Handle<Image>,
    pub height2: Handle<Image>,
    pub edge1: Handle<Image>,
    pub edge2: Handle<Image>,
    pub micro_normal: Option<Handle<Image>>,
    pub environment: Option<Handle<Image>>,
    pub reflection: Option<Handle<Image>>,
}

impl WaterMaterialTextures {
    pub fn new(
        normal1: Handle<Image>,
        normal2: Handle<Image>,
        height1: Handle<Image>,
        height2: Handle<Image>,
        edge1: Handle<Image>,
        edge2: Handle<Image>,
    ) -> Self {
        Self {
            normal1,
            normal2,
            height1,
            height2,
            edge1,
            edge2,
            micro_normal: None,
            environment: None,
            reflection: None,
        }
    }
}

/// Builds a `WaterMaterial` from params + texture handles.
pub fn build_water_material(params: WaterMaterialParams, textures: WaterMaterialTextures) -> WaterMaterial {
    WaterMaterial {
        params,
        normal1_texture: Some(textures.normal1),
        normal2_texture: Some(textures.normal2),
        height1_texture: Some(textures.height1),
        height2_texture: Some(textures.height2),
        edge1_texture: Some(textures.edge1),
        edge2_texture: Some(textures.edge2),
        micro_normal_texture: textures.micro_normal,
        environment_texture: textures.environment,
        reflection_texture: textures.reflection,
        ..default()
    }
}
