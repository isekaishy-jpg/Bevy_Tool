//! Material scaffolding for the shader sandbox.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

const WATER_SHADER_PATH: &str = "embedded://shader_sandbox/shaders/water_material.wgsl";

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct WaterMaterialParams {
    pub surface_color: LinearRgba,
    pub volume_color: LinearRgba,
    pub rim_color: LinearRgba,
    pub rim_strength: f32,
    pub base_alpha: f32,
    pub reflection_blend: f32,
    pub micro_scale: f32,
    pub micro_strength: f32,
    pub env_strength: f32,
    pub spec_strength: f32,
    pub spec_power: f32,
    pub clearcoat_strength: f32,
    pub clearcoat_power: f32,
    pub sampler1_speed: Vec2,
    pub sampler2_speed: Vec2,
    pub sampler_scale: Vec2,
    pub sampler_mix: f32,
    pub normal_strength: f32,
    pub height_strength: f32,
    pub refraction_amount: f32,
    pub edge_size: f32,
    pub air_ior: f32,
    pub ior: f32,
    pub far_clip: f32,
    pub ssr_screen_fade: f32,
    pub ssr_strength: f32,
    pub ssr_thickness: f32,
    pub planar_reflection_strength: f32,
    pub steps: u32,
    pub fog_underwater: u32,
    pub foam_or_fade: u32,
    pub debug_view: u32,
    pub lighting_enabled: u32,
    pub _padding: u32,
    pub reflection_clip_from_world: Mat4,
}

impl Default for WaterMaterialParams {
    fn default() -> Self {
        Self {
            surface_color: LinearRgba::new(0.0, 0.35, 0.65, 1.0),
            volume_color: LinearRgba::new(0.0, 0.2, 0.35, 1.0),
            rim_color: LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            rim_strength: 0.0,
            base_alpha: 0.7,
            reflection_blend: 0.6,
            micro_scale: 0.0,
            micro_strength: 0.0,
            env_strength: 0.0,
            spec_strength: 0.3,
            spec_power: 96.0,
            clearcoat_strength: 0.0,
            clearcoat_power: 256.0,
            sampler1_speed: Vec2::new(0.02, 0.0),
            sampler2_speed: Vec2::new(0.0, 0.02),
            sampler_scale: Vec2::splat(0.1),
            sampler_mix: 0.5,
            normal_strength: 1.0,
            height_strength: 0.12,
            refraction_amount: 0.08,
            edge_size: 0.1,
            air_ior: 1.0,
            ior: 1.33,
            far_clip: 50.0,
            ssr_screen_fade: 0.05,
            ssr_strength: 1.0,
            ssr_thickness: 0.04,
            planar_reflection_strength: 0.0,
            steps: 512,
            fog_underwater: 1,
            foam_or_fade: 0,
            debug_view: 0,
            lighting_enabled: 1,
            _padding: 0,
            reflection_clip_from_world: Mat4::IDENTITY,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub params: WaterMaterialParams,
    #[texture(1)]
    #[sampler(2)]
    pub normal1_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub normal2_texture: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub height1_texture: Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    pub height2_texture: Option<Handle<Image>>,
    #[texture(9)]
    #[sampler(10)]
    pub edge1_texture: Option<Handle<Image>>,
    #[texture(11)]
    #[sampler(12)]
    pub edge2_texture: Option<Handle<Image>>,
    #[texture(15)]
    #[sampler(16)]
    pub micro_normal_texture: Option<Handle<Image>>,
    #[texture(17)]
    #[sampler(18)]
    pub reflection_texture: Option<Handle<Image>>,
    #[texture(13)]
    #[sampler(14)]
    pub environment_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            params: WaterMaterialParams::default(),
            normal1_texture: None,
            normal2_texture: None,
            height1_texture: None,
            height2_texture: None,
            edge1_texture: None,
            edge2_texture: None,
            micro_normal_texture: None,
            reflection_texture: None,
            environment_texture: None,
            alpha_mode: AlphaMode::Blend,
        }
    }
}

impl Material for WaterMaterial {
    fn vertex_shader() -> ShaderRef {
        WATER_SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        WATER_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn reads_view_transmission_texture(&self) -> bool {
        true
    }

    fn enable_prepass() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
