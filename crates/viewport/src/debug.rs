use bevy::math::primitives::InfinitePlane3d;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{EditorViewportCamera, ViewportService};

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportDebugSettings {
    pub show_ray_hit_marker: bool,
}

pub fn draw_viewport_ray_hit_marker(
    debug: Res<ViewportDebugSettings>,
    service: Res<ViewportService>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorViewportCamera>>,
    mut gizmos: Gizmos,
) {
    if !debug.show_ray_hit_marker {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(ray) = service.viewport_ray(cursor, camera, camera_transform) else {
        return;
    };
    let plane = InfinitePlane3d::new(Vec3::Y);
    if let Some(hit) = ray.plane_intersection_point(Vec3::ZERO, plane) {
        gizmos.sphere(hit + Vec3::Y * 0.02, 0.12, Color::srgb(1.0, 0.85, 0.25));
    }
}
