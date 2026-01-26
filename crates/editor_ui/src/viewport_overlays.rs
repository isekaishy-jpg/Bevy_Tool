use std::path::PathBuf;

use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use editor_core::command_registry::OverlayState;
use editor_core::editor_state::{ProjectEditorStateResource, ViewportOverlayPrefs};
use viewport::{
    OverlayPresentMode, SnapKind, ViewportInputState, ViewportOverlayMaster,
    ViewportOverlaySettings,
};

#[derive(Resource, Default)]
pub struct ViewportOverlaySyncState {
    pub last_root: Option<PathBuf>,
    pub pending_prefs: Option<ViewportOverlayPrefs>,
}

pub fn handle_overlay_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    input_state: Res<ViewportInputState>,
    mut overlays: ResMut<OverlayState>,
    mut settings: ResMut<ViewportOverlaySettings>,
) {
    if !input_state.hotkeys_allowed {
        return;
    }

    if keys.just_pressed(KeyCode::KeyO) {
        overlays.show_overlays = !overlays.show_overlays;
    }

    if keys.just_pressed(KeyCode::Comma) {
        settings.cycle_snap(-1);
    }
    if keys.just_pressed(KeyCode::Period) {
        settings.cycle_snap(1);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        settings.cycle_subgrid_spacing(-1);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        settings.cycle_subgrid_spacing(1);
    }
}

pub fn sync_overlay_master_state(
    overlays: Res<OverlayState>,
    mut master: ResMut<ViewportOverlayMaster>,
) {
    if !overlays.is_changed() {
        return;
    }
    master.enabled = overlays.show_overlays;
}

pub fn sync_overlay_settings(
    mut editor_state: ResMut<ProjectEditorStateResource>,
    mut overlays: ResMut<ViewportOverlaySettings>,
    mut sync_state: ResMut<ViewportOverlaySyncState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let root = editor_state.root.clone();
    if root != sync_state.last_root {
        sync_state.last_root = root;
        sync_state.pending_prefs = None;
        *overlays = settings_from_prefs(&editor_state.state.viewport_overlays);
        return;
    }

    if overlays.is_changed() {
        overlays.normalize_subgrid_spacing();
        sync_state.pending_prefs = Some(prefs_from_settings(&overlays));
    } else if editor_state.is_changed() && sync_state.pending_prefs.is_none() {
        let desired = settings_from_prefs(&editor_state.state.viewport_overlays);
        if *overlays != desired {
            *overlays = desired;
        }
    }

    if let Some(pending) = sync_state.pending_prefs.clone() {
        if !mouse_buttons.pressed(MouseButton::Left) {
            editor_state.state.viewport_overlays = pending;
            sync_state.pending_prefs = None;
        }
    }
}

fn settings_from_prefs(prefs: &ViewportOverlayPrefs) -> ViewportOverlaySettings {
    let mut settings = ViewportOverlaySettings {
        show_cursor_readout: prefs.show_cursor_readout,
        show_fps: prefs.show_fps,
        present_mode: present_mode_from_pref(prefs.present_mode),
        show_tile_grid: prefs.show_tile_grid,
        show_chunk_grid: prefs.show_chunk_grid,
        show_subgrid: prefs.show_subgrid,
        show_region_bounds: prefs.show_region_bounds,
        show_hover_highlight: prefs.show_hover_highlight,
        show_selection_highlight: prefs.show_selection_highlight,
        show_debug_markers: prefs.show_debug_markers,
        show_streaming: prefs.show_streaming,
        snap_kind: snap_kind_from_pref(prefs.snap_mode),
        subgrid_spacing: prefs.subgrid_spacing,
    };
    settings.normalize_subgrid_spacing();
    settings
}

fn prefs_from_settings(settings: &ViewportOverlaySettings) -> ViewportOverlayPrefs {
    ViewportOverlayPrefs {
        show_cursor_readout: settings.show_cursor_readout,
        show_fps: settings.show_fps,
        present_mode: present_mode_to_pref(settings.present_mode),
        show_tile_grid: settings.show_tile_grid,
        show_chunk_grid: settings.show_chunk_grid,
        show_subgrid: settings.show_subgrid,
        show_region_bounds: settings.show_region_bounds,
        show_hover_highlight: settings.show_hover_highlight,
        show_selection_highlight: settings.show_selection_highlight,
        show_debug_markers: settings.show_debug_markers,
        show_streaming: settings.show_streaming,
        snap_mode: snap_mode_from_kind(settings.snap_kind),
        subgrid_spacing: settings.subgrid_spacing,
    }
}

fn snap_kind_from_pref(value: u8) -> SnapKind {
    match value {
        1 => SnapKind::Tile,
        2 => SnapKind::Chunk,
        3 => SnapKind::Subgrid,
        _ => SnapKind::Off,
    }
}

fn snap_mode_from_kind(kind: SnapKind) -> u8 {
    match kind {
        SnapKind::Off => 0,
        SnapKind::Tile => 1,
        SnapKind::Chunk => 2,
        SnapKind::Subgrid => 3,
    }
}

fn present_mode_from_pref(value: u8) -> OverlayPresentMode {
    match value {
        1 => OverlayPresentMode::AutoNoVsync,
        2 => OverlayPresentMode::Immediate,
        _ => OverlayPresentMode::Vsync,
    }
}

fn present_mode_to_pref(mode: OverlayPresentMode) -> u8 {
    match mode {
        OverlayPresentMode::Vsync => 0,
        OverlayPresentMode::AutoNoVsync => 1,
        OverlayPresentMode::Immediate => 2,
    }
}
