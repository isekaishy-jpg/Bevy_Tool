use bevy::prelude::*;

use crate::WorldCursor;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportDebugSettings {
    pub show_ray_hit_marker: bool,
    pub show_prop_debug: bool,
}

pub fn draw_viewport_ray_hit_marker(
    debug: Res<ViewportDebugSettings>,
    cursor: Res<WorldCursor>,
    mut gizmos: Gizmos,
) {
    if !debug.show_ray_hit_marker {
        return;
    }
    if !cursor.has_hit {
        return;
    }
    gizmos.sphere(
        cursor.hit_pos_world + Vec3::Y * 0.02,
        0.12,
        Color::srgb(1.0, 0.85, 0.25),
    );
}
