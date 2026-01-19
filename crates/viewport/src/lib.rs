//! Viewport abstraction: camera, picking, gizmos.

use bevy::prelude::*;

pub mod camera;

pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_viewport);
    }
}

fn setup_viewport(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 6.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, -0.8, 0.0)),
    ));
}
