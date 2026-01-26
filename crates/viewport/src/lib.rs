//! Viewport abstraction: camera, picking, gizmos.

use bevy::camera::{visibility::RenderLayers, ClearColorConfig, Viewport};
use bevy::gizmos::prelude::{DefaultGizmoConfigGroup, GizmoConfigStore};
use bevy::prelude::*;

pub mod camera;
mod coords;
mod cursor;
mod debug;
pub mod input;
mod overlays;
mod props;
pub mod service;
mod spatial_overlays;

pub use camera::{
    update_viewport_camera, CameraTuning, ViewportCameraController, ViewportCameraMode,
    ViewportFocusRequest, ViewportGoToTile, ViewportWorldSettings,
};
pub use cursor::{
    update_world_cursor, SnapKind, ViewportRegionBounds, ViewportRegionContext, WorldCursor,
};
pub use debug::{draw_viewport_ray_hit_marker, ViewportDebugSettings};
pub use input::{
    log_viewport_capture_changes, update_viewport_input, ViewportCaptureChanged,
    ViewportCaptureRequest, ViewportCaptureSource, ViewportInputState, ViewportUiInput,
};
pub use overlays::{
    apply_present_mode_from_overlay, subgrid_spacing_meters, OverlayPresentMode,
    TileStreamingStatus, TileStreamingVisual, ViewportOverlayMaster, ViewportOverlayScope,
    ViewportOverlaySettings, ViewportOverlayStats, ViewportSelectionState,
    ViewportTileStreamingState, SUBGRID_SPACING_LEVELS,
};
pub use props::{
    draw_debug_prop_gizmo, sync_debug_prop_visibility, update_prop_hover, DebugPropMarker,
    PropBounds, PropHoverState, PropPickable,
};
pub use service::{ViewportBackend, ViewportRect, ViewportService};
pub use spatial_overlays::{
    draw_chunk_grid_overlay, draw_hover_highlight_overlay, draw_region_bounds_overlay,
    draw_selection_highlight_overlay, draw_streaming_overlay, draw_subgrid_overlay,
    draw_tile_grid_overlay, update_overlay_scope,
};

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
            .init_resource::<ViewportRegionContext>()
            .init_resource::<WorldCursor>()
            .init_resource::<PropHoverState>()
            .init_resource::<ViewportDebugSettings>()
            .init_resource::<ViewportOverlayMaster>()
            .init_resource::<ViewportOverlaySettings>()
            .init_resource::<ViewportOverlayStats>()
            .init_resource::<ViewportOverlayScope>()
            .init_resource::<ViewportSelectionState>()
            .init_resource::<ViewportTileStreamingState>()
            .add_message::<ViewportCaptureRequest>()
            .add_message::<ViewportCaptureChanged>()
            .add_message::<ViewportFocusRequest>()
            .add_message::<ViewportGoToTile>()
            .add_systems(Startup, setup_viewport)
            .add_systems(
                PostUpdate,
                (
                    apply_camera_viewport,
                    update_viewport_input,
                    log_viewport_capture_changes.after(update_viewport_input),
                    update_viewport_camera.after(update_viewport_input),
                    update_world_cursor.after(update_viewport_camera),
                    apply_present_mode_from_overlay.after(update_world_cursor),
                    update_overlay_scope.after(update_world_cursor),
                    sync_debug_prop_visibility.after(update_world_cursor),
                    update_prop_hover.after(update_world_cursor),
                    draw_tile_grid_overlay.after(update_overlay_scope),
                    draw_chunk_grid_overlay.after(draw_tile_grid_overlay),
                    draw_subgrid_overlay.after(draw_chunk_grid_overlay),
                    draw_region_bounds_overlay.after(draw_subgrid_overlay),
                    draw_hover_highlight_overlay.after(draw_region_bounds_overlay),
                    draw_selection_highlight_overlay.after(draw_hover_highlight_overlay),
                    draw_streaming_overlay.after(draw_selection_highlight_overlay),
                    draw_debug_prop_gizmo.after(draw_streaming_overlay),
                    draw_viewport_ray_hit_marker.after(draw_debug_prop_gizmo),
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

    // TODO(04.x): Gate debug prop marker behind a dev-only toggle/feature once real props exist.
    commands.spawn((
        Transform::from_xyz(2.0, 0.5, 2.0),
        GlobalTransform::default(),
        Visibility::Hidden,
        PropPickable {
            instance_id: foundation::ids::InstanceId(1),
            bounds: PropBounds::from_half_extents(Vec3::splat(0.5)),
        },
        DebugPropMarker,
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
