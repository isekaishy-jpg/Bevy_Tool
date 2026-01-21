//! Viewport abstraction: camera, picking, gizmos.

use bevy::camera::{visibility::RenderLayers, ClearColorConfig, Viewport};
use bevy::gizmos::prelude::{DefaultGizmoConfigGroup, GizmoConfigStore};
use bevy::prelude::*;

pub mod camera;
pub mod grid;
pub mod input;
pub mod service;

pub use camera::{
    update_viewport_camera, CameraTuning, ViewportCameraController, ViewportCameraMode,
    ViewportGoToTile, ViewportWorldSettings,
};
pub use grid::draw_ground_grid;
pub use input::{
    log_viewport_capture_changes, update_viewport_input, ViewportCaptureChanged,
    ViewportCaptureRequest, ViewportCaptureSource, ViewportInputState, ViewportUiInput,
};
pub use service::{ViewportBackend, ViewportRect, ViewportService};

#[derive(Component)]
pub struct EditorViewportCamera;

pub struct ViewportPlugin;

const VIEWPORT_GIZMO_LAYER: usize = 1;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportRect>()
            .init_resource::<ViewportService>()
            .init_resource::<ViewportUiInput>()
            .init_resource::<ViewportInputState>()
            .init_resource::<ViewportCameraController>()
            .init_resource::<ViewportCameraMode>()
            .init_resource::<ViewportWorldSettings>()
            .add_message::<ViewportCaptureRequest>()
            .add_message::<ViewportCaptureChanged>()
            .add_message::<ViewportGoToTile>()
            .add_systems(Startup, setup_viewport)
            .add_systems(
                PostUpdate,
                (
                    apply_camera_viewport,
                    update_viewport_input,
                    log_viewport_capture_changes.after(update_viewport_input),
                    update_viewport_camera.after(update_viewport_input),
                    draw_ground_grid.after(update_viewport_camera),
                ),
            );
    }
}

fn setup_viewport(mut commands: Commands, gizmo_store: Option<ResMut<GizmoConfigStore>>) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.05, 0.06)),
            ..Default::default()
        },
        EditorViewportCamera,
        RenderLayers::default().with(VIEWPORT_GIZMO_LAYER),
        Transform::from_xyz(-6.0, 6.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, -0.8, 0.0)),
    ));

    if let Some(mut gizmo_store) = gizmo_store {
        let (config, _) = gizmo_store.config_mut::<DefaultGizmoConfigGroup>();
        config.render_layers = RenderLayers::layer(VIEWPORT_GIZMO_LAYER);
    }
}

fn apply_camera_viewport(
    rect: Res<ViewportRect>,
    service: Res<ViewportService>,
    mut query: Query<&mut Camera, With<EditorViewportCamera>>,
) {
    if service.backend != ViewportBackend::CameraViewport {
        return;
    }
    let mut cameras = query.iter_mut();
    let Some(mut camera) = cameras.next() else {
        return;
    };
    if rect.is_valid {
        camera.viewport = Some(Viewport {
            physical_position: rect.physical_origin,
            physical_size: rect.physical_size,
            ..Default::default()
        });
    } else {
        camera.viewport = None;
    }
}
