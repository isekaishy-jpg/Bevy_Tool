//! Egui panels, docking, widgets.

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use viewport::update_prop_hover;

pub mod panels;
pub mod selection;
pub mod viewport_overlays;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        })
        .add_plugins(EguiPlugin::default())
        .init_resource::<panels::DockLayout>()
        .init_resource::<panels::ProjectPanelState>()
        .init_resource::<panels::CommandPaletteState>()
        .init_resource::<panels::LogPanelState>()
        .init_resource::<panels::GoToTileState>()
        .init_resource::<selection::SelectionInputState>()
        .init_resource::<panels::viewport_overlay_options::ViewportOverlayPanelState>()
        .init_resource::<panels::viewport_overlay_hud::ViewportOverlayHudState>()
        .init_resource::<viewport_overlays::ViewportOverlaySyncState>()
        .add_systems(Startup, setup_ui_camera)
        .add_systems(EguiPrimaryContextPass, panels::draw_root_panel)
        .add_systems(
            PostUpdate,
            (
                panels::sync_viewport_ui_input.before(viewport::update_viewport_input),
                viewport_overlays::handle_overlay_hotkeys.after(viewport::update_viewport_input),
                viewport_overlays::sync_overlay_master_state
                    .after(viewport_overlays::handle_overlay_hotkeys)
                    .before(viewport::update_overlay_scope),
                viewport_overlays::sync_overlay_settings
                    .after(viewport_overlays::handle_overlay_hotkeys)
                    .before(viewport::update_world_cursor),
                selection::update_viewport_selection.after(update_prop_hover),
                selection::sync_viewport_selection_overlay
                    .after(selection::update_viewport_selection),
            ),
        );
    }
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..Default::default()
        },
        PrimaryEguiContext,
    ));
}
