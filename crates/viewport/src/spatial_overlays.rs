use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use foundation::ids::TileCoord;

use crate::camera::ViewportCameraController;
use crate::{
    subgrid_spacing_meters, TileStreamingVisual, ViewportCameraMode, ViewportOverlayMaster,
    ViewportOverlayScope, ViewportOverlaySettings, ViewportOverlayStats, ViewportRegionBounds,
    ViewportRegionContext, ViewportSelectionState, ViewportTileStreamingState,
    ViewportWorldSettings, WorldCursor,
};

const OVERLAY_Y: f32 = 0.02;
const FIXED_TILE_WINDOW_TILES: i32 = 4;

#[derive(SystemParam)]
pub struct OverlayScopeParams<'w> {
    master: Res<'w, ViewportOverlayMaster>,
    controller: Res<'w, ViewportCameraController>,
    mode: Res<'w, ViewportCameraMode>,
    world: Res<'w, ViewportWorldSettings>,
    region: Res<'w, ViewportRegionContext>,
    scope: ResMut<'w, ViewportOverlayScope>,
    stats: ResMut<'w, ViewportOverlayStats>,
}

pub fn update_overlay_scope(mut params: OverlayScopeParams) {
    params.stats.reset();
    params.scope.valid = false;

    if !params.master.enabled {
        return;
    }

    let tile_size = params.world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    let reference_distance = match *params.mode {
        ViewportCameraMode::Orbit => params.controller.distance,
        ViewportCameraMode::FreeFly => params.controller.position.y.abs().max(1.0),
    };
    let radius = (reference_distance * 2.0).clamp(tile_size * 1.5, tile_size * 6.0);
    let center = Vec2::new(params.controller.position.x, params.controller.position.z);

    let mut min_x = ((center.x - radius) / tile_size).floor() as i32;
    let mut max_x = ((center.x + radius) / tile_size).floor() as i32;
    let mut min_y = ((center.y - radius) / tile_size).floor() as i32;
    let mut max_y = ((center.y + radius) / tile_size).floor() as i32;

    if let Some(bounds) = params.region.bounds {
        min_x = min_x.max(bounds.min_x);
        max_x = max_x.min(bounds.max_x);
        min_y = min_y.max(bounds.min_y);
        max_y = max_y.min(bounds.max_y);
    }

    if min_x > max_x || min_y > max_y {
        return;
    }

    params.scope.valid = true;
    params.scope.min_x = min_x;
    params.scope.max_x = max_x;
    params.scope.min_y = min_y;
    params.scope.max_y = max_y;

    params.stats.tiles_considered = 0;
}

pub fn draw_tile_grid_overlay(
    master: Res<ViewportOverlayMaster>,
    settings: Res<ViewportOverlaySettings>,
    scope: Res<ViewportOverlayScope>,
    world: Res<ViewportWorldSettings>,
    region: Res<ViewportRegionContext>,
    mut stats: ResMut<ViewportOverlayStats>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !settings.show_tile_grid {
        return;
    }
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    let (tile_min_x, tile_max_x, tile_min_y, tile_max_y) = if let Some(bounds) = region.bounds {
        (bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y)
    } else if scope.valid {
        (scope.min_x, scope.max_x, scope.min_y, scope.max_y)
    } else {
        return;
    };

    stats.add_tiles(tile_count_from_range(
        tile_min_x, tile_max_x, tile_min_y, tile_max_y,
    ));

    let min_x = tile_min_x as f32 * tile_size;
    let max_x = (tile_max_x + 1) as f32 * tile_size;
    let min_z = tile_min_y as f32 * tile_size;
    let max_z = (tile_max_y + 1) as f32 * tile_size;

    let lines_x = (tile_max_x - tile_min_x + 2).max(0) as u32;
    let lines_z = (tile_max_y - tile_min_y + 2).max(0) as u32;
    stats.add_lines(lines_x + lines_z);

    let color = Color::srgba(0.28, 0.36, 0.42, 0.7);
    let mut x = min_x;
    while x <= max_x + 0.0001 {
        gizmos.line(
            Vec3::new(x, OVERLAY_Y, min_z),
            Vec3::new(x, OVERLAY_Y, max_z),
            color,
        );
        x += tile_size;
    }

    let mut z = min_z;
    while z <= max_z + 0.0001 {
        gizmos.line(
            Vec3::new(min_x, OVERLAY_Y, z),
            Vec3::new(max_x, OVERLAY_Y, z),
            color,
        );
        z += tile_size;
    }
}

#[derive(SystemParam)]
pub struct ChunkGridOverlayParams<'w> {
    master: Res<'w, ViewportOverlayMaster>,
    settings: Res<'w, ViewportOverlaySettings>,
    world: Res<'w, ViewportWorldSettings>,
    region: Res<'w, ViewportRegionContext>,
    controller: Res<'w, ViewportCameraController>,
    selection: Res<'w, ViewportSelectionState>,
    stats: ResMut<'w, ViewportOverlayStats>,
}

pub fn draw_chunk_grid_overlay(mut params: ChunkGridOverlayParams, mut gizmos: Gizmos) {
    if !params.master.enabled || !params.settings.show_chunk_grid {
        return;
    }
    let tile_size = params.world.tile_size_meters;
    let chunks = params.world.chunks_per_tile;
    if !tile_size.is_finite() || tile_size <= 0.0 || chunks == 0 {
        return;
    }
    let chunks = chunks as i32;
    let chunk_size = tile_size / chunks as f32;
    if !chunk_size.is_finite() || chunk_size <= 0.0 {
        return;
    }

    let Some((tile_min_x, tile_max_x, tile_min_y, tile_max_y)) = fixed_tile_window(
        &params.controller,
        &params.world,
        params.region.bounds,
        params.selection.selected_tile,
    ) else {
        return;
    };

    params.stats.add_tiles(tile_count_from_range(
        tile_min_x, tile_max_x, tile_min_y, tile_max_y,
    ));

    let min_x = tile_min_x as f32 * tile_size;
    let max_x = (tile_max_x + 1) as f32 * tile_size;
    let min_z = tile_min_y as f32 * tile_size;
    let max_z = (tile_max_y + 1) as f32 * tile_size;

    let tiles_x = tile_max_x - tile_min_x + 1;
    let tiles_z = tile_max_y - tile_min_y + 1;
    let lines_x = tiles_x.saturating_mul(chunks) + 1;
    let lines_z = tiles_z.saturating_mul(chunks) + 1;
    params.stats.add_lines(lines_x as u32 + lines_z as u32);

    let color = Color::srgba(0.23, 0.3, 0.34, 0.6);
    let mut x = min_x;
    while x <= max_x + 0.0001 {
        gizmos.line(
            Vec3::new(x, OVERLAY_Y, min_z),
            Vec3::new(x, OVERLAY_Y, max_z),
            color,
        );
        x += chunk_size;
    }

    let mut z = min_z;
    while z <= max_z + 0.0001 {
        gizmos.line(
            Vec3::new(min_x, OVERLAY_Y, z),
            Vec3::new(max_x, OVERLAY_Y, z),
            color,
        );
        z += chunk_size;
    }
}

#[derive(SystemParam)]
pub struct SubgridOverlayParams<'w> {
    master: Res<'w, ViewportOverlayMaster>,
    settings: Res<'w, ViewportOverlaySettings>,
    world: Res<'w, ViewportWorldSettings>,
    region: Res<'w, ViewportRegionContext>,
    controller: Res<'w, ViewportCameraController>,
    selection: Res<'w, ViewportSelectionState>,
    mode: Res<'w, ViewportCameraMode>,
    stats: ResMut<'w, ViewportOverlayStats>,
}

pub fn draw_subgrid_overlay(mut params: SubgridOverlayParams, mut gizmos: Gizmos) {
    if !params.master.enabled || !params.settings.show_subgrid {
        return;
    }
    let tile_size = params.world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    let altitude = match *params.mode {
        ViewportCameraMode::Orbit => {
            (params.controller.distance * params.controller.pitch.sin()).abs()
        }
        ViewportCameraMode::FreeFly => params.controller.position.y.abs(),
    };
    let near_threshold = tile_size * 3.0;
    if altitude > near_threshold {
        return;
    }

    let spacing = subgrid_spacing_meters(&params.settings, &params.world);

    let Some((tile_min_x, tile_max_x, tile_min_y, tile_max_y)) = fixed_tile_window(
        &params.controller,
        &params.world,
        params.region.bounds,
        params.selection.selected_tile,
    ) else {
        return;
    };

    params.stats.add_tiles(tile_count_from_range(
        tile_min_x, tile_max_x, tile_min_y, tile_max_y,
    ));

    let min_world_x = tile_min_x as f32 * tile_size;
    let max_world_x = (tile_max_x + 1) as f32 * tile_size;
    let min_world_z = tile_min_y as f32 * tile_size;
    let max_world_z = (tile_max_y + 1) as f32 * tile_size;

    let (origin_x, origin_z) = region_origin(params.region.bounds, tile_size);
    let (start_idx_x, count_x) = aligned_line_indices(min_world_x, max_world_x, origin_x, spacing);
    let (start_idx_z, count_z) = aligned_line_indices(min_world_z, max_world_z, origin_z, spacing);
    params.stats.add_lines(count_x as u32 + count_z as u32);

    let color = Color::srgba(0.19, 0.24, 0.28, 0.45);
    for i in 0..count_x {
        let x = origin_x + (start_idx_x + i as i32) as f32 * spacing;
        gizmos.line(
            Vec3::new(x, OVERLAY_Y, min_world_z),
            Vec3::new(x, OVERLAY_Y, max_world_z),
            color,
        );
    }
    for i in 0..count_z {
        let z = origin_z + (start_idx_z + i as i32) as f32 * spacing;
        gizmos.line(
            Vec3::new(min_world_x, OVERLAY_Y, z),
            Vec3::new(max_world_x, OVERLAY_Y, z),
            color,
        );
    }
}

pub fn draw_region_bounds_overlay(
    master: Res<ViewportOverlayMaster>,
    settings: Res<ViewportOverlaySettings>,
    world: Res<ViewportWorldSettings>,
    region: Res<ViewportRegionContext>,
    mut stats: ResMut<ViewportOverlayStats>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !settings.show_region_bounds {
        return;
    }
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }
    let Some(bounds) = region.bounds else {
        return;
    };

    let min = Vec3::new(
        bounds.min_x as f32 * tile_size,
        OVERLAY_Y,
        bounds.min_y as f32 * tile_size,
    );
    let max = Vec3::new(
        (bounds.max_x + 1) as f32 * tile_size,
        OVERLAY_Y,
        (bounds.max_y + 1) as f32 * tile_size,
    );
    stats.add_lines(4);
    draw_rect(&mut gizmos, min, max, Color::srgba(0.8, 0.25, 0.25, 0.8));
}

pub fn draw_hover_highlight_overlay(
    master: Res<ViewportOverlayMaster>,
    settings: Res<ViewportOverlaySettings>,
    cursor: Res<WorldCursor>,
    world: Res<ViewportWorldSettings>,
    mut stats: ResMut<ViewportOverlayStats>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !settings.show_hover_highlight {
        return;
    }
    if !cursor.has_hit || !cursor.in_bounds {
        return;
    }
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    let tile_min = Vec3::new(
        cursor.tile_x as f32 * tile_size,
        OVERLAY_Y + 0.02,
        cursor.tile_y as f32 * tile_size,
    );
    let tile_max = Vec3::new(
        (cursor.tile_x + 1) as f32 * tile_size,
        OVERLAY_Y + 0.02,
        (cursor.tile_y + 1) as f32 * tile_size,
    );
    stats.add_lines(4);
    draw_rect(
        &mut gizmos,
        tile_min,
        tile_max,
        Color::srgba(0.95, 0.85, 0.25, 0.9),
    );

    if settings.show_chunk_grid {
        let chunks = world.chunks_per_tile as f32;
        if chunks > 0.0 {
            let chunk_size = tile_size / chunks;
            let chunk_min = Vec3::new(
                cursor.tile_x as f32 * tile_size + cursor.chunk_x as f32 * chunk_size,
                OVERLAY_Y + 0.04,
                cursor.tile_y as f32 * tile_size + cursor.chunk_y as f32 * chunk_size,
            );
            let chunk_max = Vec3::new(
                chunk_min.x + chunk_size,
                OVERLAY_Y + 0.04,
                chunk_min.z + chunk_size,
            );
            stats.add_lines(4);
            draw_rect(
                &mut gizmos,
                chunk_min,
                chunk_max,
                Color::srgba(0.3, 0.9, 0.95, 0.85),
            );
        }
    }
}

pub fn draw_selection_highlight_overlay(
    master: Res<ViewportOverlayMaster>,
    settings: Res<ViewportOverlaySettings>,
    selection: Res<ViewportSelectionState>,
    world: Res<ViewportWorldSettings>,
    mut stats: ResMut<ViewportOverlayStats>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !settings.show_selection_highlight {
        return;
    }
    let Some(tile) = selection.selected_tile else {
        return;
    };
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    let min = Vec3::new(
        tile.x as f32 * tile_size,
        OVERLAY_Y + 0.06,
        tile.y as f32 * tile_size,
    );
    let max = Vec3::new(
        (tile.x + 1) as f32 * tile_size,
        OVERLAY_Y + 0.06,
        (tile.y + 1) as f32 * tile_size,
    );
    stats.add_lines(4);
    draw_rect(&mut gizmos, min, max, Color::srgba(0.25, 0.75, 0.95, 0.95));
}

pub fn draw_streaming_overlay(
    master: Res<ViewportOverlayMaster>,
    settings: Res<ViewportOverlaySettings>,
    scope: Res<ViewportOverlayScope>,
    world: Res<ViewportWorldSettings>,
    streaming: Res<ViewportTileStreamingState>,
    mut stats: ResMut<ViewportOverlayStats>,
    mut gizmos: Gizmos,
) {
    if !master.enabled || !settings.show_streaming || !scope.valid {
        return;
    }
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }

    stats.add_tiles(scope.tile_count());
    for tile_x in scope.min_x..=scope.max_x {
        for tile_y in scope.min_y..=scope.max_y {
            let tile = foundation::ids::TileCoord {
                x: tile_x,
                y: tile_y,
            };
            let Some(state) = streaming
                .tile_state(tile)
                .and_then(|status| status.visual_state())
            else {
                continue;
            };

            let min = Vec3::new(
                tile_x as f32 * tile_size,
                OVERLAY_Y + 0.01,
                tile_y as f32 * tile_size,
            );
            let max = Vec3::new(
                (tile_x + 1) as f32 * tile_size,
                OVERLAY_Y + 0.01,
                (tile_y + 1) as f32 * tile_size,
            );
            let color = streaming_color(state);
            stats.add_lines(4);
            draw_rect(&mut gizmos, min, max, color);
            if matches!(state, TileStreamingVisual::Error) {
                stats.add_lines(2);
                draw_cross(&mut gizmos, min, max, color);
            }
        }
    }
}

fn region_origin(bounds: Option<ViewportRegionBounds>, tile_size: f32) -> (f32, f32) {
    if let Some(bounds) = bounds {
        (
            bounds.min_x as f32 * tile_size,
            bounds.min_y as f32 * tile_size,
        )
    } else {
        (0.0, 0.0)
    }
}

fn tile_count_from_range(min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> u32 {
    let width = (max_x - min_x + 1).max(0) as u32;
    let height = (max_y - min_y + 1).max(0) as u32;
    width.saturating_mul(height)
}

fn aligned_line_indices(min: f32, max: f32, origin: f32, spacing: f32) -> (i32, usize) {
    if !spacing.is_finite() || spacing <= 0.0 {
        return (0, 0);
    }
    let start = ((min - origin) / spacing).floor() as i32;
    let end = ((max - origin) / spacing).ceil() as i32;
    let count = (end - start + 1).max(0) as usize;
    (start, count)
}

fn fixed_tile_window(
    controller: &ViewportCameraController,
    world: &ViewportWorldSettings,
    bounds: Option<ViewportRegionBounds>,
    selected_tile: Option<TileCoord>,
) -> Option<(i32, i32, i32, i32)> {
    let tile_size = world.tile_size_meters;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return None;
    }
    let center = if let Some(tile) = selected_tile {
        (tile.x, tile.y)
    } else {
        tile_coord_from_world(
            Vec2::new(controller.position.x, controller.position.z),
            tile_size,
        )?
    };
    let window = FIXED_TILE_WINDOW_TILES.max(1);
    let min_x = center.0 - window / 2;
    let min_y = center.1 - window / 2;
    let mut max_x = min_x + window - 1;
    let mut max_y = min_y + window - 1;
    let mut min_x = min_x;
    let mut min_y = min_y;

    if let Some(bounds) = bounds {
        min_x = min_x.max(bounds.min_x);
        max_x = max_x.min(bounds.max_x);
        min_y = min_y.max(bounds.min_y);
        max_y = max_y.min(bounds.max_y);
    }

    if min_x > max_x || min_y > max_y {
        return None;
    }

    Some((min_x, max_x, min_y, max_y))
}

fn tile_coord_from_world(position: Vec2, tile_size_meters: f32) -> Option<(i32, i32)> {
    if !tile_size_meters.is_finite() || tile_size_meters <= 0.0 {
        return None;
    }
    let tile_x = (position.x / tile_size_meters).floor() as i32;
    let tile_y = (position.y / tile_size_meters).floor() as i32;
    Some((tile_x, tile_y))
}

fn streaming_color(state: TileStreamingVisual) -> Color {
    match state {
        TileStreamingVisual::Loaded => Color::srgba(0.2, 0.7, 0.25, 0.6),
        TileStreamingVisual::PendingLoad => Color::srgba(0.95, 0.8, 0.2, 0.7),
        TileStreamingVisual::Dirty => Color::srgba(0.95, 0.5, 0.2, 0.75),
        TileStreamingVisual::Pinned => Color::srgba(0.3, 0.7, 0.95, 0.7),
        TileStreamingVisual::Error => Color::srgba(0.95, 0.2, 0.2, 0.9),
    }
}

fn draw_rect(gizmos: &mut Gizmos, min: Vec3, max: Vec3, color: Color) {
    let a = Vec3::new(min.x, min.y, min.z);
    let b = Vec3::new(max.x, min.y, min.z);
    let c = Vec3::new(max.x, min.y, max.z);
    let d = Vec3::new(min.x, min.y, max.z);
    gizmos.line(a, b, color);
    gizmos.line(b, c, color);
    gizmos.line(c, d, color);
    gizmos.line(d, a, color);
}

fn draw_cross(gizmos: &mut Gizmos, min: Vec3, max: Vec3, color: Color) {
    let a = Vec3::new(min.x, min.y, min.z);
    let b = Vec3::new(max.x, min.y, max.z);
    let c = Vec3::new(max.x, min.y, min.z);
    let d = Vec3::new(min.x, min.y, max.z);
    gizmos.line(a, b, color);
    gizmos.line(c, d, color);
}
