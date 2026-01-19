//! Preview helpers for the shader sandbox.

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::core_pipeline::Skybox;
use bevy::ecs::message::MessageReader;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseMotion;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::primitives::Plane3d;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use bevy::camera::{visibility::RenderLayers, RenderTarget};

use crate::material::{WaterMaterial, WaterMaterialParams};

pub struct WaterDebugPlugin;

impl Plugin for WaterDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterDebugState>()
            .add_systems(Startup, log_debug_controls)
            .add_systems(Update, water_debug_controls);
    }
}

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, log_camera_controls).add_systems(
            Update,
            (
                camera_controls,
                sync_reflection_camera.after(camera_controls),
            ),
        );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct WaterDebugState {
    initialized: bool,
    ssr_enabled: bool,
    refraction_enabled: bool,
    fog_underwater: bool,
    foam_or_fade: bool,
    debug_view: u32,
    ssr_strength: f32,
    refraction_amount: f32,
    steps: u32,
}

impl Default for WaterDebugState {
    fn default() -> Self {
        let params = WaterMaterialParams::default();
        Self {
            initialized: false,
            ssr_enabled: params.ssr_strength > 0.0,
            refraction_enabled: params.refraction_amount > 0.0,
            fog_underwater: params.fog_underwater != 0,
            foam_or_fade: params.foam_or_fade != 0,
            debug_view: params.debug_view,
            ssr_strength: params.ssr_strength,
            refraction_amount: params.refraction_amount,
            steps: clamp_steps(params.steps),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct FlyCamera {
    speed: f32,
    boost: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
    initialized: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

#[derive(Component, Debug, Clone, Copy)]
pub struct ReflectionCamera;

#[derive(Resource, Debug, Clone)]
pub struct WaterPreviewState {
    water_material: Handle<WaterMaterial>,
    water_height: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            speed: 6.0,
            boost: 3.0,
            sensitivity: 0.003,
            yaw: 0.0,
            pitch: 0.0,
            initialized: false,
        }
    }
}

const MIN_SSR_STEPS: u32 = 64;
const MAX_SSR_STEPS: u32 = 1024;
const SSR_STEP_DELTA: u32 = 64;
const SSR_STEP_QUANTIZE: u32 = 16;
const SSR_STRENGTH_STEP: f32 = 0.1;
const SSR_STRENGTH_MAX: f32 = 2.0;
const REFRACTION_STEP: f32 = 0.02;
const REFRACTION_MAX: f32 = 0.5;

fn log_debug_controls() {
    info!("Water debug controls: 1=SSR 2=Refraction 3=Fog 4=Foam/Fade  5=SSR Debug  [ ]=Steps  -=SSR  +=SSR  ,=Refract  .=Refract");
}

fn log_camera_controls() {
    info!("Camera controls: WASD+QE move, Shift boost, RMB look");
}

fn water_debug_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WaterDebugState>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    query: Query<&MeshMaterial3d<WaterMaterial>>,
) {
    if !state.initialized {
        if let Some(handle) = query.iter().next() {
            if let Some(material) = materials.get(&handle.0) {
                sync_debug_state(&mut state, material.params);
            }
        }
    }

    let mut dirty = false;

    if keys.just_pressed(KeyCode::Digit1) {
        state.ssr_enabled = !state.ssr_enabled;
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.refraction_enabled = !state.refraction_enabled;
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.fog_underwater = !state.fog_underwater;
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        state.foam_or_fade = !state.foam_or_fade;
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        state.debug_view = (state.debug_view + 1) % 3;
        dirty = true;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        state.steps = clamp_steps(state.steps.saturating_sub(SSR_STEP_DELTA));
        dirty = true;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        state.steps = clamp_steps(state.steps.saturating_add(SSR_STEP_DELTA));
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Minus) {
        state.ssr_strength = (state.ssr_strength - SSR_STRENGTH_STEP).max(0.0);
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Equal) {
        state.ssr_strength = (state.ssr_strength + SSR_STRENGTH_STEP).min(SSR_STRENGTH_MAX);
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Comma) {
        state.refraction_amount = (state.refraction_amount - REFRACTION_STEP).max(0.0);
        dirty = true;
    }
    if keys.just_pressed(KeyCode::Period) {
        state.refraction_amount = (state.refraction_amount + REFRACTION_STEP).min(REFRACTION_MAX);
        dirty = true;
    }

    if !dirty {
        return;
    }

    for handle in query.iter() {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.params.ssr_strength = if state.ssr_enabled {
                state.ssr_strength
            } else {
                0.0
            };
            material.params.refraction_amount = if state.refraction_enabled {
                state.refraction_amount
            } else {
                0.0
            };
            material.params.fog_underwater = if state.fog_underwater { 1 } else { 0 };
            material.params.foam_or_fade = if state.foam_or_fade { 1 } else { 0 };
            material.params.debug_view = state.debug_view;
            material.params.steps = clamp_steps(state.steps);
        }
    }

    info!(
        "Water debug: ssr={} refraction={} fog={} foam={} debug={} steps={} ssr_strength={:.2} refract={:.2}",
        state.ssr_enabled,
        state.refraction_enabled,
        state.fog_underwater,
        state.foam_or_fade,
        state.debug_view,
        state.steps,
        state.ssr_strength,
        state.refraction_amount
    );
}

fn camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_delta += motion.delta;
    }

    for (mut transform, mut camera) in query.iter_mut() {
        if !camera.initialized {
            let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            camera.yaw = yaw;
            camera.pitch = pitch;
            camera.initialized = true;
        }

        if mouse_buttons.pressed(MouseButton::Right) {
            camera.yaw -= mouse_delta.x * camera.sensitivity;
            camera.pitch = (camera.pitch - mouse_delta.y * camera.sensitivity).clamp(-1.54, 1.54);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
        }

        let mut input = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            input.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            input.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            input.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyQ) {
            input.y -= 1.0;
        }

        if input == Vec3::ZERO {
            continue;
        }

        let speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            camera.speed * camera.boost
        } else {
            camera.speed
        };

        let forward = transform.forward();
        let right = transform.right();
        let up = Vec3::Y;
        let direction = (forward * input.z + right * input.x + up * input.y).normalize();
        transform.translation += direction * speed * time.delta_secs();
    }
}

fn sync_reflection_camera(
    state: Option<Res<WaterPreviewState>>,
    main_camera: Query<(&GlobalTransform, &Projection), (With<MainCamera>, Without<ReflectionCamera>)>,
    mut reflection_camera: Query<(&mut Transform, &mut Projection), With<ReflectionCamera>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let Some(state) = state else {
        return;
    };
    let Ok((main_transform, main_projection)) = main_camera.single() else {
        return;
    };
    let Ok((mut reflection_transform, mut reflection_projection)) = reflection_camera.single_mut()
    else {
        return;
    };

    let main = main_transform.compute_transform();
    let normal = Vec3::Y;
    let reflected_position = Vec3::new(
        main.translation.x,
        state.water_height * 2.0 - main.translation.y,
        main.translation.z,
    );
    let forward = main.forward().as_vec3();
    let up = main.up().as_vec3();
    let reflected_forward = forward - 2.0 * forward.dot(normal) * normal;
    let reflected_up = up - 2.0 * up.dot(normal) * normal;

    *reflection_transform = Transform::from_translation(reflected_position)
        .looking_at(reflected_position + reflected_forward, reflected_up);
    *reflection_projection = main_projection.clone();

    if let Some(material) = materials.get_mut(&state.water_material) {
        let view = reflection_transform.to_matrix().inverse();
        let clip_from_world = reflection_projection.get_clip_from_view() * view;
        material.params.reflection_clip_from_world = clip_from_world;
    }
}

fn clamp_steps(steps: u32) -> u32 {
    let clamped = steps.clamp(MIN_SSR_STEPS, MAX_SSR_STEPS);
    (clamped / SSR_STEP_QUANTIZE) * SSR_STEP_QUANTIZE
}

fn sync_debug_state(state: &mut WaterDebugState, params: WaterMaterialParams) {
    state.initialized = true;
    state.ssr_enabled = params.ssr_strength > 0.0;
    state.refraction_enabled = params.refraction_amount > 0.0;
    state.fog_underwater = params.fog_underwater != 0;
    state.foam_or_fade = params.foam_or_fade != 0;
    state.debug_view = params.debug_view;
    state.ssr_strength = params.ssr_strength;
    state.refraction_amount = params.refraction_amount;
    state.steps = clamp_steps(params.steps);
}

pub fn setup_preview_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.4, 0.0)),
    ));

    let water_height = -0.25;
    let water_mesh = Plane3d::default()
        .mesh()
        .size(20.0, 20.0)
        .subdivisions(64)
        .build();

    let height_a_data = generate_wave_height(256, Vec2::new(9.0, 7.0), Vec2::new(-11.0, 5.0));
    let height_b_data = generate_wave_height(256, Vec2::new(13.0, -9.0), Vec2::new(-7.0, -12.0));
    let normal_a = images.add(make_normal_from_height(256, &height_a_data, 6.0, true));
    let normal_b = images.add(make_normal_from_height(256, &height_b_data, 6.0, true));
    let height_a = images.add(make_height_image(256, &height_a_data, true));
    let height_b = images.add(make_height_image(256, &height_b_data, true));
    let micro_height_data = generate_noise_height(64, 1337);
    let micro_normal = images.add(make_normal_from_height(64, &micro_height_data, 10.0, true));
    let edge_a = images.add(make_solid_image([255, 255, 255, 255], true));
    let edge_b = images.add(make_checker_image(8, 200, 255, true));
    let environment_map = images.add(make_environment_map(256, 128));
    let reflection_texture = images.add(make_reflection_target(512, 512));
    let skybox_texture = images.add(make_skybox_image([70, 90, 120, 255]));

    let params = WaterMaterialParams {
        height_strength: 0.12,
        normal_strength: 2.6,
        sampler_scale: Vec2::splat(0.22),
        refraction_amount: 0.15,
        edge_size: 5.0,
        foam_or_fade: 1,
        steps: 0,
        ssr_screen_fade: 0.0,
        ssr_strength: 0.0,
        ssr_thickness: 0.06,
        planar_reflection_strength: 0.0,
        base_alpha: 0.55,
        surface_color: LinearRgba::new(0.02, 0.08, 0.14, 1.0),
        volume_color: LinearRgba::new(0.04, 0.18, 0.26, 1.0),
        reflection_blend: 1.2,
        micro_scale: 16.0,
        micro_strength: 0.03,
        rim_strength: 0.0,
        env_strength: 0.5,
        spec_strength: 0.7,
        spec_power: 64.0,
        clearcoat_strength: 0.6,
        clearcoat_power: 320.0,
        lighting_enabled: 1,
        ..Default::default()
    };
    info!(
        "Water params: alpha={:.2} ssr_steps={} ssr_strength={:.2} ssr_thickness={:.2} planar_strength={:.2} micro_scale={:.2} micro_strength={:.2} env_strength={:.2} spec_strength={:.2} spec_power={:.1} clearcoat_strength={:.2} clearcoat_power={:.1}",
        params.base_alpha,
        params.steps,
        params.ssr_strength,
        params.ssr_thickness,
        params.planar_reflection_strength,
        params.micro_scale,
        params.micro_strength,
        params.env_strength,
        params.spec_strength,
        params.spec_power,
        params.clearcoat_strength,
        params.clearcoat_power
    );

    let water_material = materials.add(WaterMaterial {
        params,
        normal1_texture: Some(normal_a),
        normal2_texture: Some(normal_b),
        height1_texture: Some(height_a),
        height2_texture: Some(height_b),
        edge1_texture: Some(edge_a),
        edge2_texture: Some(edge_b),
        micro_normal_texture: Some(micro_normal),
        reflection_texture: Some(reflection_texture.clone()),
        environment_texture: Some(environment_map),
        ..default()
    });
    commands.insert_resource(WaterPreviewState {
        water_material: water_material.clone(),
        water_height,
    });

    commands.spawn((
        Mesh3d(meshes.add(water_mesh)),
        MeshMaterial3d(water_material),
        Transform::from_xyz(0.0, water_height, 0.0),
        RenderLayers::layer(0),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        RenderTarget::default(),
        DepthPrepass,
        Transform::from_xyz(-12.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCamera::default(),
        MainCamera,
        RenderLayers::layer(0),
        Skybox {
            image: skybox_texture.clone(),
            brightness: 800.0,
            ..default()
        },
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            is_active: false,
            ..default()
        },
        RenderTarget::Image(reflection_texture.into()),
        Transform::from_xyz(-12.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ReflectionCamera,
        RenderLayers::layer(1),
        Skybox {
            image: skybox_texture,
            brightness: 800.0,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb_u8(220, 60, 60),
            perceptual_roughness: 0.4,
            metallic: 0.05,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.25, 0.0),
        RenderLayers::from_layers(&[0, 1]),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0).build())),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb_u8(40, 45, 50),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -1.75, 0.0),
        RenderLayers::from_layers(&[0, 1]),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 2.0))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb_u8(80, 140, 230),
            perceptual_roughness: 0.15,
            metallic: 0.65,
            ..default()
        })),
        Transform::from_xyz(3.0, 1.5, -2.0),
        RenderLayers::from_layers(&[0, 1]),
    ));
}

fn make_solid_image(color: [u8; 4], repeat: bool) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &color,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    if repeat {
        set_repeat_linear(&mut image);
    }
    image
}

fn make_checker_image(size: u32, low: u8, high: u8, repeat: bool) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let value = if (x + y) % 2 == 0 { high } else { low };
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );

    if repeat {
        set_repeat_linear(&mut image);
    }

    image
}

fn generate_wave_height(size: u32, frequency_a: Vec2, frequency_b: Vec2) -> Vec<f32> {
    let mut data = Vec::with_capacity((size * size) as usize);
    let tau = std::f32::consts::TAU;
    let (rot_sin, rot_cos) = 0.65_f32.sin_cos();

    for y in 0..size {
        let v = y as f32 / size as f32;
        for x in 0..size {
            let u = x as f32 / size as f32;
            let warp = (tau * (u * 1.7 + v * 1.3)).sin() * 0.02
                + (tau * (u * 2.3 - v * 1.1)).sin() * 0.015;
            let u_w = u + warp;
            let v_w = v - warp * 0.8;
            let u_r = u_w * rot_cos - v_w * rot_sin;
            let v_r = u_w * rot_sin + v_w * rot_cos;

            let wave_a = (tau * (u_w * frequency_a.x + v_w * frequency_a.y + 0.37)).sin();
            let wave_b = (tau * (u_r * frequency_b.x + v_r * frequency_b.y - 1.12)).sin();
            let wave_c =
                (tau * (u_w * frequency_a.x * 2.1 + v_w * frequency_b.y * 1.8 + 2.4)).sin();
            let wave_d =
                (tau * (u_r * frequency_b.x * 1.6 + v_r * frequency_a.y * 2.0 - 0.7)).sin();
            let height = (wave_a + wave_b) * 0.35 + (wave_c + wave_d) * 0.2;
            data.push(height * 0.5 + 0.5);
        }
    }

    data
}

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

fn generate_noise_height(size: u32, seed: u32) -> Vec<f32> {
    let mut data = Vec::with_capacity((size * size) as usize);
    let last = size.saturating_sub(1);

    for y in 0..size {
        let ty = if y == last { 0 } else { y };
        for x in 0..size {
            let tx = if x == last { 0 } else { x };
            let hash = hash_u32(tx ^ (ty << 16) ^ seed);
            let value = hash as f32 / u32::MAX as f32;
            data.push(value);
        }
    }

    data
}

fn make_height_image(size: u32, heights: &[f32], repeat: bool) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for height in heights {
        let value = (height * 255.0).clamp(0.0, 255.0) as u8;
        data.extend_from_slice(&[value, value, value, 255]);
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );

    if repeat {
        set_repeat_linear(&mut image);
    }

    image
}

fn make_normal_from_height(size: u32, heights: &[f32], strength: f32, repeat: bool) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let left = ((x - 1 + size as i32) % size as i32) as usize;
            let right = ((x + 1) % size as i32) as usize;
            let down = ((y - 1 + size as i32) % size as i32) as usize;
            let up = ((y + 1) % size as i32) as usize;

            let index_left = (y as usize) * size as usize + left;
            let index_right = (y as usize) * size as usize + right;
            let index_down = down * size as usize + x as usize;
            let index_up = up * size as usize + x as usize;

            let dx = (heights[index_right] - heights[index_left]) * strength;
            let dy = (heights[index_up] - heights[index_down]) * strength;
            let normal = Vec3::new(-dx, -dy, 1.0).normalize();

            let r = (normal.x * 0.5 + 0.5).clamp(0.0, 1.0);
            let g = (normal.y * 0.5 + 0.5).clamp(0.0, 1.0);
            let b = (normal.z * 0.5 + 0.5).clamp(0.0, 1.0);
            data.extend_from_slice(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );

    if repeat {
        set_repeat_linear(&mut image);
    }

    image
}

fn set_repeat_linear(image: &mut Image) {
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
}

fn set_clamp_linear(image: &mut Image) {
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
}

fn make_skybox_image(color: [u8; 4]) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 6,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &color,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image
        .reinterpret_stacked_2d_as_array(6)
        .expect("skybox image should be 6 layers");
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

fn make_reflection_target(width: u32, height: u32) -> Image {
    let mut image = Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);
    set_clamp_linear(&mut image);
    image
}

fn make_environment_map(width: u32, height: u32) -> Image {
    let sky = Vec3::new(0.6, 0.75, 0.95);
    let horizon = Vec3::new(0.85, 0.9, 0.95);
    let ground = Vec3::new(0.08, 0.08, 0.1);
    let sun_color = Vec3::new(1.0, 0.96, 0.85);
    let sun_u = 0.2;
    let sun_v = 0.22;
    let sun_sharpness = 450.0;
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        let t = if height > 1 {
            y as f32 / (height - 1) as f32
        } else {
            0.0
        };
        let base = sky.lerp(ground, t);
        let horizon_mix = (1.0 - (t - 0.5).abs() * 2.0).clamp(0.0, 1.0);
        let row_color = base.lerp(horizon, horizon_mix * 0.5);

        for x in 0..width {
            let u = if width > 1 {
                x as f32 / (width - 1) as f32
            } else {
                0.0
            };
            let du = u - sun_u;
            let dv = t - sun_v;
            let sun = (-sun_sharpness * (du * du + dv * dv)).exp();
            let color = row_color + sun_color * sun * 1.5;
            let r = (color.x * 255.0).clamp(0.0, 255.0) as u8;
            let g = (color.y * 255.0).clamp(0.0, 255.0) as u8;
            let b = (color.z * 255.0).clamp(0.0, 255.0) as u8;
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    set_clamp_linear(&mut image);
    image
}
