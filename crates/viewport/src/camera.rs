//! Camera control policy (v0). RTS/orbit controls for the editor viewport.

use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::input::ButtonInput;
use bevy::prelude::*;

use crate::{EditorViewportCamera, ViewportCaptureSource, ViewportInputState};

mod math;

use math::{
    camera_offset, forward_direction, forward_right, ground_focus_point, mouse_scroll_lines,
    speed_multiplier, yaw_pitch_from_forward, yaw_pitch_from_offset,
};

#[derive(Debug, Clone, Copy)]
pub struct CameraTuning {
    pub base_speed: f32,
    pub speed_altitude_scale: f32,
    pub slow_multiplier: f32,
    pub fast_multiplier: f32,
}

impl Default for CameraTuning {
    fn default() -> Self {
        Self {
            base_speed: 20.0,
            speed_altitude_scale: 0.02,
            slow_multiplier: 0.25,
            fast_multiplier: 3.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportCameraMode {
    #[default]
    Orbit,
    FreeFly,
}

impl ViewportCameraMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Orbit => "Orbit",
            Self::FreeFly => "Free Fly",
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ViewportCameraController {
    pub orbit_focus: Vec3,
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pitch_limits: Vec2,
    pub min_altitude: f32,
    pub max_distance: f32,
    pub pan_sensitivity: f32,
    pub orbit_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub tuning: CameraTuning,
    active_mode: ViewportCameraMode,
    initialized: bool,
}

impl Default for ViewportCameraController {
    fn default() -> Self {
        Self {
            orbit_focus: Vec3::ZERO,
            position: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.8,
            distance: 12.0,
            pitch_limits: Vec2::new(0.2, 1.45),
            min_altitude: 2.0,
            max_distance: 20000.0,
            pan_sensitivity: 0.002,
            orbit_sensitivity: 0.005,
            zoom_sensitivity: 35.0,
            tuning: CameraTuning::default(),
            active_mode: ViewportCameraMode::Orbit,
            initialized: false,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportWorldSettings {
    pub tile_size_meters: f32,
    pub chunks_per_tile: u16,
}

impl Default for ViewportWorldSettings {
    fn default() -> Self {
        Self {
            tile_size_meters: 512.0,
            chunks_per_tile: 16,
        }
    }
}

impl ViewportWorldSettings {
    pub fn tile_center(self, tile_x: i32, tile_y: i32) -> Vec3 {
        let tile_size = self.tile_size_meters;
        Vec3::new(
            (tile_x as f32 + 0.5) * tile_size,
            0.0,
            (tile_y as f32 + 0.5) * tile_size,
        )
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportGoToTile {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ViewportFocusRequest {
    pub world_point: Option<Vec3>,
}

#[derive(SystemParam)]
pub struct ViewportCameraInputs<'w, 's> {
    time: Res<'w, Time>,
    input_state: Res<'w, ViewportInputState>,
    world_settings: Res<'w, ViewportWorldSettings>,
    mode: Res<'w, ViewportCameraMode>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_motion: MessageReader<'w, 's, MouseMotion>,
    mouse_wheel: MessageReader<'w, 's, MouseWheel>,
    go_to_tile: MessageReader<'w, 's, ViewportGoToTile>,
    focus_requests: MessageReader<'w, 's, ViewportFocusRequest>,
}

pub fn update_viewport_camera(
    mut inputs: ViewportCameraInputs,
    mut controller: ResMut<ViewportCameraController>,
    mut query: Query<&mut Transform, With<EditorViewportCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    if !controller.initialized {
        initialize_controller(&mut controller, &transform);
    }

    let mode = *inputs.mode;
    // Tool capture disables camera input to avoid conflicts with active tools.
    let tool_captured = inputs.input_state.captured
        && inputs.input_state.captor == Some(ViewportCaptureSource::Tool);
    if controller.active_mode != mode {
        apply_mode_transition(mode, &mut controller, &transform);
        controller.active_mode = mode;
    }

    let mut mouse_delta = Vec2::ZERO;
    for motion in inputs.mouse_motion.read() {
        mouse_delta += motion.delta;
    }

    let mut scroll_lines = 0.0;
    for wheel in inputs.mouse_wheel.read() {
        scroll_lines += mouse_scroll_lines(*wheel);
    }

    if let Some(target) = last_go_to_tile(&mut inputs.go_to_tile) {
        let mut destination = inputs.world_settings.tile_center(target.0, target.1);
        match mode {
            ViewportCameraMode::Orbit => controller.orbit_focus = destination,
            ViewportCameraMode::FreeFly => {
                destination.y = controller.position.y;
                controller.position = destination;
            }
        }
    }

    if let Some(request) = last_focus_request(&mut inputs.focus_requests) {
        apply_focus_request(request, mode, &mut controller, &transform);
    }

    let altitude = current_altitude(mode, &controller);

    let hotkeys_allowed = inputs.input_state.hotkeys_allowed && !tool_captured;
    if hotkeys_allowed {
        match mode {
            ViewportCameraMode::Orbit => apply_keyboard_pan(
                &inputs.keys,
                inputs.time.delta_secs(),
                altitude,
                &mut controller,
            ),
            ViewportCameraMode::FreeFly => apply_keyboard_fly(
                &inputs.keys,
                inputs.time.delta_secs(),
                altitude,
                &mut controller,
            ),
        }

        if mode == ViewportCameraMode::Orbit && inputs.keys.just_pressed(KeyCode::KeyF) {
            apply_focus_request(
                ViewportFocusRequest { world_point: None },
                mode,
                &mut controller,
                &transform,
            );
        }
    }

    if inputs.input_state.focused && !tool_captured {
        match mode {
            ViewportCameraMode::Orbit => {
                let alt_pressed =
                    inputs.keys.pressed(KeyCode::AltLeft) || inputs.keys.pressed(KeyCode::AltRight);
                if alt_pressed
                    && inputs.input_state.captured
                    && inputs.mouse_buttons.pressed(MouseButton::Left)
                {
                    apply_orbit_look(mouse_delta, &mut controller);
                } else if inputs.mouse_buttons.pressed(MouseButton::Middle) {
                    apply_mouse_pan(
                        mouse_delta,
                        controller.yaw,
                        controller.pan_sensitivity,
                        controller.distance,
                        &mut controller.orbit_focus,
                    );
                }

                if scroll_lines.abs() > f32::EPSILON {
                    apply_zoom(scroll_lines, &inputs.keys, altitude, &mut controller);
                }
            }
            ViewportCameraMode::FreeFly => {
                if inputs.mouse_buttons.pressed(MouseButton::Right) {
                    apply_free_look(mouse_delta, &mut controller);
                } else if inputs.mouse_buttons.pressed(MouseButton::Middle) {
                    apply_mouse_pan(
                        mouse_delta,
                        controller.yaw,
                        controller.pan_sensitivity,
                        altitude.max(1.0),
                        &mut controller.position,
                    );
                }

                if scroll_lines.abs() > f32::EPSILON {
                    apply_free_scroll(scroll_lines, &inputs.keys, altitude, &mut controller);
                }
            }
        }
    }

    clamp_controller(&mut controller, mode);
    apply_camera_transform(&mut controller, mode, &mut transform);
}

fn initialize_controller(controller: &mut ViewportCameraController, transform: &Transform) {
    controller.position = transform.translation;
    let offset = controller.position - controller.orbit_focus;
    let (yaw, pitch) = yaw_pitch_from_offset(offset);
    let limits = pitch_limits_for_mode(ViewportCameraMode::Orbit, controller);
    controller.yaw = yaw;
    controller.pitch = pitch.clamp(limits.x, limits.y);
    controller.distance = offset.length().max(controller.min_distance());
    let offset = camera_offset(controller.yaw, controller.pitch, controller.distance);
    controller.orbit_focus = controller.position - offset;
    controller.initialized = true;
}

fn apply_mode_transition(
    mode: ViewportCameraMode,
    controller: &mut ViewportCameraController,
    transform: &Transform,
) {
    controller.position = transform.translation;

    match mode {
        ViewportCameraMode::Orbit => {
            let mut distance = controller.distance;
            if !distance.is_finite() || distance <= f32::EPSILON {
                distance = (controller.position - controller.orbit_focus).length();
            }
            if distance <= f32::EPSILON {
                distance = 12.0;
            }
            let forward = transform.forward().as_vec3().normalize_or_zero();
            let offset = -forward * distance;
            let (yaw, pitch) = yaw_pitch_from_offset(offset);
            let limits = pitch_limits_for_mode(mode, controller);
            controller.yaw = yaw;
            controller.pitch = pitch.clamp(limits.x, limits.y);
            controller.distance = distance.max(controller.min_distance());
            let offset = camera_offset(controller.yaw, controller.pitch, controller.distance);
            controller.orbit_focus = controller.position - offset;
        }
        ViewportCameraMode::FreeFly => {
            let (yaw, pitch) = yaw_pitch_from_forward(transform.forward().as_vec3());
            let limits = pitch_limits_for_mode(mode, controller);
            controller.yaw = yaw;
            controller.pitch = pitch.clamp(limits.x, limits.y);
        }
    }
}

fn apply_focus_request(
    request: ViewportFocusRequest,
    mode: ViewportCameraMode,
    controller: &mut ViewportCameraController,
    transform: &Transform,
) {
    if mode != ViewportCameraMode::Orbit {
        return;
    }
    let target = request
        .world_point
        .or_else(|| ground_focus_point(transform));
    let Some(target) = target else {
        return;
    };

    controller.orbit_focus = target;
    let offset = controller.position - target;
    if offset.length_squared() <= f32::EPSILON {
        return;
    }
    let (yaw, pitch) = yaw_pitch_from_offset(offset);
    let limits = pitch_limits_for_mode(ViewportCameraMode::Orbit, controller);
    controller.yaw = yaw;
    controller.pitch = pitch.clamp(limits.x, limits.y);
    controller.distance = offset.length().max(controller.min_distance());
}

fn current_altitude(mode: ViewportCameraMode, controller: &ViewportCameraController) -> f32 {
    match mode {
        ViewportCameraMode::Orbit => controller.orbit_altitude(),
        ViewportCameraMode::FreeFly => controller.position.y.abs(),
    }
}

fn pitch_limits_for_mode(mode: ViewportCameraMode, controller: &ViewportCameraController) -> Vec2 {
    match mode {
        ViewportCameraMode::Orbit => controller.pitch_limits,
        ViewportCameraMode::FreeFly => Vec2::new(-1.54, 1.54),
    }
}

fn movement_axis(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let mut axis = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        axis.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        axis.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        axis.x += 1.0;
    }
    axis
}

fn apply_keyboard_pan(
    keys: &ButtonInput<KeyCode>,
    dt: f32,
    altitude: f32,
    controller: &mut ViewportCameraController,
) {
    let axis = movement_axis(keys);
    if axis.length_squared() <= f32::EPSILON {
        return;
    }

    let (forward, right) = forward_right(controller.yaw);
    let direction = (forward * axis.y + right * axis.x).normalize_or_zero();
    let speed = controller.tuning.base_speed
        * speed_multiplier(keys, &controller.tuning)
        * (1.0 + altitude * controller.tuning.speed_altitude_scale);
    controller.orbit_focus += direction * speed * dt;
}

fn apply_keyboard_fly(
    keys: &ButtonInput<KeyCode>,
    dt: f32,
    altitude: f32,
    controller: &mut ViewportCameraController,
) {
    let axis = movement_axis(keys);
    if axis.length_squared() <= f32::EPSILON {
        return;
    }
    let forward = forward_direction(controller.yaw, controller.pitch).normalize_or_zero();
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let direction = (forward * axis.y + right * axis.x).normalize_or_zero();
    let speed = controller.tuning.base_speed
        * speed_multiplier(keys, &controller.tuning)
        * (1.0 + altitude * controller.tuning.speed_altitude_scale);
    controller.position += direction * speed * dt;
}

fn apply_orbit_look(delta: Vec2, controller: &mut ViewportCameraController) {
    controller.yaw -= delta.x * controller.orbit_sensitivity;
    let limits = pitch_limits_for_mode(ViewportCameraMode::Orbit, controller);
    controller.pitch =
        (controller.pitch + delta.y * controller.orbit_sensitivity).clamp(limits.x, limits.y);
}

fn apply_free_look(delta: Vec2, controller: &mut ViewportCameraController) {
    controller.yaw -= delta.x * controller.orbit_sensitivity;
    let limits = pitch_limits_for_mode(ViewportCameraMode::FreeFly, controller);
    controller.pitch =
        (controller.pitch - delta.y * controller.orbit_sensitivity).clamp(limits.x, limits.y);
}

fn apply_mouse_pan(delta: Vec2, yaw: f32, pan_sensitivity: f32, scale: f32, target: &mut Vec3) {
    if delta.length_squared() <= f32::EPSILON {
        return;
    }
    let (forward, right) = forward_right(yaw);
    let pan = (-right * delta.x + forward * delta.y) * pan_sensitivity * scale;
    *target += pan;
}

fn apply_zoom(
    scroll_lines: f32,
    keys: &ButtonInput<KeyCode>,
    altitude: f32,
    controller: &mut ViewportCameraController,
) {
    let speed = controller.zoom_sensitivity
        * speed_multiplier(keys, &controller.tuning)
        * (1.0 + altitude * controller.tuning.speed_altitude_scale);
    controller.distance =
        (controller.distance - scroll_lines * speed).clamp(0.1, controller.max_distance);
}

fn apply_free_scroll(
    scroll_lines: f32,
    keys: &ButtonInput<KeyCode>,
    altitude: f32,
    controller: &mut ViewportCameraController,
) {
    let speed = controller.zoom_sensitivity
        * speed_multiplier(keys, &controller.tuning)
        * (1.0 + altitude * controller.tuning.speed_altitude_scale);
    let forward = forward_direction(controller.yaw, controller.pitch).normalize_or_zero();
    controller.position += forward * scroll_lines * speed;
}

fn clamp_controller(controller: &mut ViewportCameraController, mode: ViewportCameraMode) {
    let limits = pitch_limits_for_mode(mode, controller);
    controller.pitch = controller.pitch.clamp(limits.x, limits.y);
    if mode == ViewportCameraMode::Orbit {
        let min_distance = controller.min_distance();
        if controller.distance < min_distance {
            controller.distance = min_distance;
        }
    }
}

fn apply_camera_transform(
    controller: &mut ViewportCameraController,
    mode: ViewportCameraMode,
    transform: &mut Transform,
) {
    match mode {
        ViewportCameraMode::Orbit => {
            let offset = camera_offset(controller.yaw, controller.pitch, controller.distance);
            transform.translation = controller.orbit_focus + offset;
            transform.look_at(controller.orbit_focus, Vec3::Y);
            controller.position = transform.translation;
        }
        ViewportCameraMode::FreeFly => {
            let forward = forward_direction(controller.yaw, controller.pitch).normalize_or_zero();
            transform.translation = controller.position;
            transform.look_at(controller.position + forward, Vec3::Y);
            controller.position = transform.translation;
        }
    }
}

impl ViewportCameraController {
    fn orbit_altitude(&self) -> f32 {
        (self.distance * self.pitch.sin()).abs()
    }

    fn min_distance(&self) -> f32 {
        let sin_pitch = self.pitch.sin().max(0.05);
        (self.min_altitude / sin_pitch).max(0.1)
    }
}

fn last_go_to_tile(reader: &mut MessageReader<ViewportGoToTile>) -> Option<(i32, i32)> {
    let mut last = None;
    for request in reader.read() {
        last = Some((request.tile_x, request.tile_y));
    }
    last
}

fn last_focus_request(
    reader: &mut MessageReader<ViewportFocusRequest>,
) -> Option<ViewportFocusRequest> {
    let mut last = None;
    for request in reader.read() {
        last = Some(*request);
    }
    last
}
