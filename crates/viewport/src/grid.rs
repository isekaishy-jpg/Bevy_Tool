use bevy::prelude::*;

use crate::{ViewportCameraController, ViewportWorldSettings};

pub fn draw_ground_grid(
    mut gizmos: Gizmos,
    controller: Res<ViewportCameraController>,
    world: Res<ViewportWorldSettings>,
) {
    let spacing = (world.tile_size_meters / 128.0).clamp(1.0, 32.0);
    let half_extent = (controller.distance * 2.0).clamp(20.0, world.tile_size_meters * 4.0);
    let grid_color = Color::srgba(0.28, 0.36, 0.42, 0.9);
    let axis_x = Color::srgba(0.95, 0.25, 0.2, 0.9);
    let axis_z = Color::srgba(0.2, 0.55, 0.95, 0.9);

    let center = Vec3::new(controller.position.x, 0.0, controller.position.z);
    let min_x = center.x - half_extent;
    let max_x = center.x + half_extent;
    let min_z = center.z - half_extent;
    let max_z = center.z + half_extent;

    let start_x = (min_x / spacing).floor() * spacing;
    let end_x = (max_x / spacing).ceil() * spacing;
    let start_z = (min_z / spacing).floor() * spacing;
    let end_z = (max_z / spacing).ceil() * spacing;

    let mut x = start_x;
    while x <= end_x + 0.0001 {
        let color = if x.abs() <= spacing * 0.5 {
            axis_x
        } else {
            grid_color
        };
        gizmos.line(Vec3::new(x, 0.0, min_z), Vec3::new(x, 0.0, max_z), color);
        x += spacing;
    }

    let mut z = start_z;
    while z <= end_z + 0.0001 {
        let color = if z.abs() <= spacing * 0.5 {
            axis_z
        } else {
            grid_color
        };
        gizmos.line(Vec3::new(min_x, 0.0, z), Vec3::new(max_x, 0.0, z), color);
        z += spacing;
    }
}
