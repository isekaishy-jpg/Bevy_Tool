use bevy::math::primitives::InfinitePlane3d;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{EditorViewportCamera, ViewportInputState, ViewportService, ViewportWorldSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    Off,
    Tile,
    Chunk,
    Subgrid,
}

#[derive(Resource, Debug, Clone)]
pub struct WorldCursor {
    pub has_hit: bool,
    pub hit_pos_world: Vec3,
    pub hit_normal_world: Vec3,
    pub region_id: Option<String>,
    pub region_name: Option<String>,
    pub tile_x: i32,
    pub tile_y: i32,
    pub chunk_x: u16,
    pub chunk_y: u16,
    pub snap_pos_world: Vec3,
    pub snap_kind: SnapKind,
}

impl Default for WorldCursor {
    fn default() -> Self {
        Self {
            has_hit: false,
            hit_pos_world: Vec3::ZERO,
            hit_normal_world: Vec3::Y,
            region_id: None,
            region_name: None,
            tile_x: 0,
            tile_y: 0,
            chunk_x: 0,
            chunk_y: 0,
            snap_pos_world: Vec3::ZERO,
            snap_kind: SnapKind::Off,
        }
    }
}

impl WorldCursor {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportRegionBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl ViewportRegionBounds {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn contains_tile(&self, tile_x: i32, tile_y: i32) -> bool {
        tile_x >= self.min_x && tile_x <= self.max_x && tile_y >= self.min_y && tile_y <= self.max_y
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ViewportRegionContext {
    pub region_id: Option<String>,
    pub region_name: Option<String>,
    pub bounds: Option<ViewportRegionBounds>,
}

impl ViewportRegionContext {
    pub fn clear(&mut self) {
        self.region_id = None;
        self.region_name = None;
        self.bounds = None;
    }
}

pub fn update_world_cursor(
    mut cursor: ResMut<WorldCursor>,
    service: Res<ViewportService>,
    input_state: Res<ViewportInputState>,
    world_settings: Res<ViewportWorldSettings>,
    region: Res<ViewportRegionContext>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorViewportCamera>>,
) {
    if !input_state.hovered {
        cursor.clear();
        return;
    }
    let Ok(window) = windows.single() else {
        cursor.clear();
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        cursor.clear();
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        cursor.clear();
        return;
    };
    let Some(ray) = service.viewport_ray(cursor_pos, camera, camera_transform) else {
        cursor.clear();
        return;
    };
    let plane = InfinitePlane3d::new(Vec3::Y);
    let Some(hit) = ray.plane_intersection_point(Vec3::ZERO, plane) else {
        cursor.clear();
        return;
    };
    if !hit.is_finite() {
        cursor.clear();
        return;
    }
    let Some((tile_x, tile_y)) = tile_coord_from_world(hit, world_settings.tile_size_meters) else {
        cursor.clear();
        return;
    };
    if let Some(bounds) = region.bounds {
        if !bounds.contains_tile(tile_x, tile_y) {
            cursor.clear();
            return;
        }
    }
    let (chunk_x, chunk_y) = chunk_coord_from_world(
        hit,
        tile_x,
        tile_y,
        world_settings.tile_size_meters,
        world_settings.chunks_per_tile,
    )
    .unwrap_or((0, 0));

    cursor.has_hit = true;
    cursor.hit_pos_world = hit;
    cursor.hit_normal_world = Vec3::Y;
    cursor.region_id = region.region_id.clone();
    cursor.region_name = region.region_name.clone();
    cursor.tile_x = tile_x;
    cursor.tile_y = tile_y;
    cursor.chunk_x = chunk_x;
    cursor.chunk_y = chunk_y;
    cursor.snap_pos_world = hit;
    cursor.snap_kind = SnapKind::Off;
}

fn tile_coord_from_world(position: Vec3, tile_size_meters: f32) -> Option<(i32, i32)> {
    if !tile_size_meters.is_finite() || tile_size_meters <= 0.0 {
        return None;
    }
    let tile_x = (position.x / tile_size_meters).floor() as i32;
    let tile_y = (position.z / tile_size_meters).floor() as i32;
    Some((tile_x, tile_y))
}

fn chunk_coord_from_world(
    position: Vec3,
    tile_x: i32,
    tile_y: i32,
    tile_size_meters: f32,
    chunks_per_tile: u16,
) -> Option<(u16, u16)> {
    if !tile_size_meters.is_finite() || tile_size_meters <= 0.0 {
        return None;
    }
    if chunks_per_tile == 0 {
        return None;
    }
    let chunk_count = chunks_per_tile as i32;
    let chunk_size = tile_size_meters / chunk_count as f32;
    if !chunk_size.is_finite() || chunk_size <= 0.0 {
        return None;
    }
    let local_x = position.x - tile_x as f32 * tile_size_meters;
    let local_z = position.z - tile_y as f32 * tile_size_meters;
    let chunk_x = (local_x / chunk_size).floor() as i32;
    let chunk_y = (local_z / chunk_size).floor() as i32;
    let max_index = chunk_count - 1;
    let chunk_x = chunk_x.clamp(0, max_index) as u16;
    let chunk_y = chunk_y.clamp(0, max_index) as u16;
    Some((chunk_x, chunk_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_coord_from_world_uses_floor() {
        let (x, y) = tile_coord_from_world(Vec3::new(511.9, 0.0, 0.1), 512.0).unwrap();
        assert_eq!((x, y), (0, 0));
        let (x, y) = tile_coord_from_world(Vec3::new(512.0, 0.0, 512.0), 512.0).unwrap();
        assert_eq!((x, y), (1, 1));
    }

    #[test]
    fn tile_coord_from_world_handles_negative() {
        let (x, y) = tile_coord_from_world(Vec3::new(-0.1, 0.0, -0.1), 512.0).unwrap();
        assert_eq!((x, y), (-1, -1));
    }

    #[test]
    fn chunk_coord_from_world_matches_tile_local_space() {
        let pos = Vec3::new(150.0, 0.0, 50.0);
        let (tile_x, tile_y) = tile_coord_from_world(pos, 100.0).unwrap();
        let (chunk_x, chunk_y) = chunk_coord_from_world(pos, tile_x, tile_y, 100.0, 4).unwrap();
        assert_eq!((tile_x, tile_y), (1, 0));
        assert_eq!((chunk_x, chunk_y), (2, 2));
    }
}
