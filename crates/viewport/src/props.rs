use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use foundation::ids::InstanceId;

use crate::{EditorViewportCamera, ViewportDebugSettings, ViewportInputState, ViewportService};

#[derive(Component, Debug, Clone, Copy)]
pub struct PropPickable {
    pub instance_id: InstanceId,
    pub bounds: PropBounds,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DebugPropMarker;

#[derive(Debug, Clone, Copy)]
pub struct PropBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl PropBounds {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_half_extents(half_extents: Vec3) -> Self {
        Self {
            min: -half_extents,
            max: half_extents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropHit {
    pub instance_id: InstanceId,
    pub world_pos: Vec3,
    pub distance: f32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PropHoverState {
    pub hovered: Option<PropHit>,
}

impl PropHoverState {
    pub fn clear(&mut self) {
        self.hovered = None;
    }
}

pub fn update_prop_hover(
    mut hover: ResMut<PropHoverState>,
    service: Res<ViewportService>,
    input_state: Res<ViewportInputState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorViewportCamera>>,
    props: Query<(&GlobalTransform, &PropPickable, &Visibility)>,
) {
    if !input_state.hovered {
        hover.clear();
        return;
    }
    let Ok(window) = windows.single() else {
        hover.clear();
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        hover.clear();
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        hover.clear();
        return;
    };
    let Some(ray) = service.viewport_ray(cursor, camera, camera_transform) else {
        hover.clear();
        return;
    };

    let mut best: Option<PropHit> = None;
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.as_vec3();

    for (transform, pickable, visibility) in props.iter() {
        if matches!(visibility, Visibility::Hidden) {
            continue;
        }
        let world_to_local = transform.to_matrix().inverse();
        if !world_to_local.is_finite() {
            continue;
        }
        let local_origin = world_to_local.transform_point3(ray_origin);
        let local_dir = world_to_local.transform_vector3(ray_dir);
        if local_dir.length_squared() <= f32::EPSILON {
            continue;
        }
        let Some(t) = ray_aabb_intersection(local_origin, local_dir, pickable.bounds) else {
            continue;
        };
        let local_hit = local_origin + local_dir * t;
        let world_hit = transform.to_matrix().transform_point3(local_hit);
        if !world_hit.is_finite() {
            continue;
        }
        let distance = (world_hit - ray_origin).length();
        if distance <= f32::EPSILON {
            continue;
        }
        let hit = PropHit {
            instance_id: pickable.instance_id,
            world_pos: world_hit,
            distance,
        };
        let replace = best
            .as_ref()
            .map(|current| distance < current.distance)
            .unwrap_or(true);
        if replace {
            best = Some(hit);
        }
    }

    hover.hovered = best;
}

pub fn sync_debug_prop_visibility(
    debug: Res<ViewportDebugSettings>,
    mut props: Query<&mut Visibility, With<DebugPropMarker>>,
) {
    if !debug.is_changed() {
        return;
    }
    let visibility = if debug.show_prop_debug {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut prop_visibility in &mut props {
        *prop_visibility = visibility;
    }
}

pub fn draw_debug_prop_gizmo(
    debug: Res<ViewportDebugSettings>,
    props: Query<(&GlobalTransform, &PropPickable), With<DebugPropMarker>>,
    mut gizmos: Gizmos,
) {
    if !debug.show_prop_debug {
        return;
    }
    let color = Color::srgb(0.95, 0.35, 0.35);
    for (transform, pickable) in &props {
        let corners = aabb_corners(pickable.bounds.min, pickable.bounds.max);
        let matrix = transform.to_matrix();
        let corners: Vec<Vec3> = corners
            .into_iter()
            .map(|corner| matrix.transform_point3(corner))
            .collect();
        for (a, b) in aabb_edges() {
            gizmos.line(corners[a], corners[b], color);
        }
    }
}

fn aabb_corners(min: Vec3, max: Vec3) -> [Vec3; 8] {
    [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ]
}

fn aabb_edges() -> [(usize, usize); 12] {
    [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ]
}

fn ray_aabb_intersection(origin: Vec3, direction: Vec3, bounds: PropBounds) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let dir_axis = direction[axis];
        let min_axis = bounds.min[axis];
        let max_axis = bounds.max[axis];

        if dir_axis.abs() <= 1e-6 {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inv = 1.0 / dir_axis;
        let mut t1 = (min_axis - origin_axis) * inv;
        let mut t2 = (max_axis - origin_axis) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_max < t_min {
            return None;
        }
    }

    if t_max < 0.0 {
        return None;
    }
    if t_min >= 0.0 {
        Some(t_min)
    } else {
        Some(t_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_aabb_hits_when_pointing_at_box() {
        let bounds = PropBounds::from_half_extents(Vec3::splat(1.0));
        let origin = Vec3::new(-3.0, 0.0, 0.0);
        let direction = Vec3::X;
        let t = ray_aabb_intersection(origin, direction, bounds).unwrap();
        assert!(t > 1.9 && t < 2.1);
    }

    #[test]
    fn ray_aabb_misses_when_parallel_outside() {
        let bounds = PropBounds::from_half_extents(Vec3::splat(1.0));
        let origin = Vec3::new(0.0, 2.0, 0.0);
        let direction = Vec3::X;
        assert!(ray_aabb_intersection(origin, direction, bounds).is_none());
    }
}
