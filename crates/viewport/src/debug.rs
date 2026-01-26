use bevy::prelude::*;

use crate::{ViewportOverlayMaster, ViewportOverlaySettings, WorldCursor};

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportDebugSettings {
    pub show_ray_hit_marker: bool,
    pub show_prop_debug: bool,
}

pub fn draw_viewport_ray_hit_marker(
    master: Res<ViewportOverlayMaster>,
    overlay_settings: Res<ViewportOverlaySettings>,
    debug: Res<ViewportDebugSettings>,
    cursor: Res<WorldCursor>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !overlay_settings.show_debug_markers || !debug.show_ray_hit_marker {
        return;
    }
    if !cursor.has_hit || !cursor.in_bounds {
        return;
    }
    gizmos.sphere(
        cursor.hit_pos_world + Vec3::Y * 0.02,
        0.12,
        Color::srgb(1.0, 0.85, 0.25),
    );
}
