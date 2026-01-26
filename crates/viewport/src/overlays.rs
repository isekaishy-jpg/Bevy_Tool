use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, Window};
use foundation::ids::{InstanceId, TileCoord};

use crate::SnapKind;

pub const SUBGRID_SPACING_LEVELS: [u16; 6] = [32, 16, 8, 4, 2, 1];

/// Master overlay gate synced from the editor command state.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportOverlayMaster {
    pub enabled: bool,
}

impl Default for ViewportOverlayMaster {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Runtime overlay settings (persisted via editor state sync).
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ViewportOverlaySettings {
    pub show_cursor_readout: bool,
    pub show_fps: bool,
    pub present_mode: OverlayPresentMode,
    pub show_tile_grid: bool,
    pub show_chunk_grid: bool,
    pub show_subgrid: bool,
    pub show_region_bounds: bool,
    pub show_hover_highlight: bool,
    pub show_selection_highlight: bool,
    pub show_debug_markers: bool,
    pub show_streaming: bool,
    pub snap_kind: SnapKind,
    pub subgrid_spacing: u16,
}

impl Default for ViewportOverlaySettings {
    fn default() -> Self {
        Self {
            show_cursor_readout: true,
            show_fps: true,
            present_mode: OverlayPresentMode::Vsync,
            show_tile_grid: true,
            show_chunk_grid: false,
            show_subgrid: false,
            show_region_bounds: true,
            show_hover_highlight: true,
            show_selection_highlight: true,
            show_debug_markers: true,
            show_streaming: false,
            snap_kind: SnapKind::Off,
            subgrid_spacing: 8,
        }
    }
}

/// Overlay-configured present mode options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPresentMode {
    Vsync,
    AutoNoVsync,
    Immediate,
}

impl OverlayPresentMode {
    pub fn as_present_mode(self) -> PresentMode {
        match self {
            OverlayPresentMode::Vsync => PresentMode::AutoVsync,
            OverlayPresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
            OverlayPresentMode::Immediate => PresentMode::Immediate,
        }
    }
}

pub fn apply_present_mode_from_overlay(
    settings: Res<ViewportOverlaySettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let desired = settings.present_mode.as_present_mode();
    if window.present_mode != desired {
        window.present_mode = desired;
    }
}

impl ViewportOverlaySettings {
    pub fn normalize_subgrid_spacing(&mut self) {
        if !SUBGRID_SPACING_LEVELS.contains(&self.subgrid_spacing) {
            self.subgrid_spacing = 8;
        }
    }

    pub fn cycle_snap(&mut self, direction: i8) {
        let order = [
            SnapKind::Off,
            SnapKind::Tile,
            SnapKind::Chunk,
            SnapKind::Subgrid,
        ];
        let index = order
            .iter()
            .position(|kind| *kind == self.snap_kind)
            .unwrap_or(0) as i8;
        let next = (index + direction).clamp(0, (order.len() - 1) as i8) as usize;
        self.snap_kind = order[next];
    }

    pub fn cycle_subgrid_spacing(&mut self, direction: i8) {
        let index = SUBGRID_SPACING_LEVELS
            .iter()
            .position(|spacing| *spacing == self.subgrid_spacing)
            .unwrap_or(2) as i8;
        let next = (index + direction).clamp(0, (SUBGRID_SPACING_LEVELS.len() - 1) as i8) as usize;
        self.subgrid_spacing = SUBGRID_SPACING_LEVELS[next];
    }
}

/// Per-frame overlay counters for performance debugging.
#[derive(Resource, Debug, Default, Clone)]
pub struct ViewportOverlayStats {
    pub lines_drawn: u32,
    pub tiles_considered: u32,
}

impl ViewportOverlayStats {
    pub fn reset(&mut self) {
        self.lines_drawn = 0;
        self.tiles_considered = 0;
    }

    pub fn add_lines(&mut self, count: u32) {
        self.lines_drawn = self.lines_drawn.saturating_add(count);
    }

    pub fn add_tiles(&mut self, count: u32) {
        self.tiles_considered = self.tiles_considered.saturating_add(count);
    }
}

/// Selection data consumed by viewport overlays.
#[derive(Resource, Debug, Default, Clone)]
pub struct ViewportSelectionState {
    pub selected_tile: Option<TileCoord>,
    pub selected_prop: Option<InstanceId>,
}

/// Convert sub-grid spacing level into meters based on world settings.
pub fn subgrid_spacing_meters(
    settings: &ViewportOverlaySettings,
    world: &crate::ViewportWorldSettings,
) -> f32 {
    let base_spacing = settings.subgrid_spacing.max(1) as f32;
    let tile_size = world.tile_size_meters;
    let chunks = world.chunks_per_tile as f32;
    if !tile_size.is_finite() || tile_size <= 0.0 || chunks <= 0.0 {
        return base_spacing;
    }
    let chunk_size = tile_size / chunks;
    if !chunk_size.is_finite() || chunk_size <= 0.0 {
        return base_spacing;
    }
    let scale = base_spacing / 32.0;
    (chunk_size * scale).max(0.01)
}

/// Cached overlay tile scope (bounded, local view).
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct ViewportOverlayScope {
    pub valid: bool,
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl ViewportOverlayScope {
    pub fn tile_count(self) -> u32 {
        if !self.valid {
            return 0;
        }
        let width = (self.max_x - self.min_x + 1).max(0) as u32;
        let height = (self.max_y - self.min_y + 1).max(0) as u32;
        width.saturating_mul(height)
    }
}

/// Streaming state per tile (fed by streaming systems later).
#[derive(Resource, Debug, Default, Clone)]
pub struct ViewportTileStreamingState {
    tiles: HashMap<TileCoord, TileStreamingStatus>,
}

impl ViewportTileStreamingState {
    pub fn set_tile_state(&mut self, tile: TileCoord, status: TileStreamingStatus) {
        if status.is_empty() {
            self.tiles.remove(&tile);
        } else {
            self.tiles.insert(tile, status);
        }
    }

    pub fn tile_state(&self, tile: TileCoord) -> Option<&TileStreamingStatus> {
        self.tiles.get(&tile)
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

/// Visualized tile streaming state after precedence is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStreamingVisual {
    Loaded,
    PendingLoad,
    Dirty,
    Pinned,
    Error,
}

/// Multi-flag tile status; visual precedence is Error > Pending > Dirty > Pinned > Loaded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileStreamingStatus {
    pub loaded: bool,
    pub pending_load: bool,
    pub dirty: bool,
    pub pinned: bool,
    pub error: bool,
}

impl TileStreamingStatus {
    pub fn is_empty(self) -> bool {
        !self.loaded && !self.pending_load && !self.dirty && !self.pinned && !self.error
    }

    pub fn visual_state(self) -> Option<TileStreamingVisual> {
        if self.error {
            Some(TileStreamingVisual::Error)
        } else if self.pending_load {
            Some(TileStreamingVisual::PendingLoad)
        } else if self.dirty {
            Some(TileStreamingVisual::Dirty)
        } else if self.pinned {
            Some(TileStreamingVisual::Pinned)
        } else if self.loaded {
            Some(TileStreamingVisual::Loaded)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_subgrid_spacing_defaults_to_eight() {
        let mut settings = ViewportOverlaySettings {
            subgrid_spacing: 7,
            ..Default::default()
        };
        settings.normalize_subgrid_spacing();
        assert_eq!(settings.subgrid_spacing, 8);
    }

    #[test]
    fn cycle_snap_clamps_bounds() {
        let mut settings = ViewportOverlaySettings::default();
        settings.cycle_snap(-1);
        assert_eq!(settings.snap_kind, SnapKind::Off);
        settings.cycle_snap(1);
        assert_eq!(settings.snap_kind, SnapKind::Tile);
        settings.cycle_snap(1);
        assert_eq!(settings.snap_kind, SnapKind::Chunk);
        settings.cycle_snap(1);
        assert_eq!(settings.snap_kind, SnapKind::Subgrid);
        settings.cycle_snap(1);
        assert_eq!(settings.snap_kind, SnapKind::Subgrid);
    }

    #[test]
    fn streaming_visual_state_uses_precedence() {
        let status = TileStreamingStatus {
            loaded: true,
            dirty: true,
            ..Default::default()
        };
        assert_eq!(status.visual_state(), Some(TileStreamingVisual::Dirty));
        let status = TileStreamingStatus {
            error: true,
            dirty: true,
            ..Default::default()
        };
        assert_eq!(status.visual_state(), Some(TileStreamingVisual::Error));
    }
}
