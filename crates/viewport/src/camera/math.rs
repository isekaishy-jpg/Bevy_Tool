use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::ButtonInput;
use bevy::prelude::{Transform, Vec3};

use super::CameraTuning;

pub fn forward_right(yaw: f32) -> (Vec3, Vec3) {
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    (forward, right)
}

pub fn forward_direction(yaw: f32, pitch: f32) -> Vec3 {
    let cos_pitch = pitch.cos();
    Vec3::new(-yaw.sin() * cos_pitch, pitch.sin(), -yaw.cos() * cos_pitch)
}

pub fn yaw_pitch_from_forward(forward: Vec3) -> (f32, f32) {
    let forward = forward.normalize_or_zero();
    if forward.length_squared() <= f32::EPSILON {
        return (0.0, 0.0);
    }
    let yaw = (-forward.x).atan2(-forward.z);
    let pitch = forward.y.asin();
    (yaw, pitch)
}

pub fn yaw_pitch_from_offset(offset: Vec3) -> (f32, f32) {
    let distance = offset.length();
    if distance <= f32::EPSILON {
        return (0.0, 0.0);
    }
    let yaw = offset.x.atan2(offset.z);
    let pitch = (offset.y / distance).asin();
    (yaw, pitch)
}

pub fn camera_offset(yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    let cos_pitch = pitch.cos();
    Vec3::new(
        distance * yaw.sin() * cos_pitch,
        distance * pitch.sin(),
        distance * yaw.cos() * cos_pitch,
    )
}

pub fn ground_focus_point(transform: &Transform) -> Option<Vec3> {
    let forward = transform.forward();
    if forward.y.abs() <= f32::EPSILON {
        return None;
    }
    let t = -transform.translation.y / forward.y;
    if t <= 0.0 {
        return None;
    }
    Some(transform.translation + forward * t)
}

pub fn mouse_scroll_lines(wheel: MouseWheel) -> f32 {
    match wheel.unit {
        MouseScrollUnit::Line => wheel.y,
        MouseScrollUnit::Pixel => wheel.y / 120.0,
    }
}

pub fn speed_multiplier(keys: &ButtonInput<KeyCode>, tuning: &CameraTuning) -> f32 {
    let fast = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let slow = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if slow {
        tuning.slow_multiplier
    } else if fast {
        tuning.fast_multiplier
    } else {
        1.0
    }
}
