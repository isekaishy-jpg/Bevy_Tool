//! Egui panels, docking, widgets.

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use viewport::update_prop_hover;

pub mod panels;
pub mod selection;

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
        .add_systems(Startup, setup_ui_camera)
        .add_systems(EguiPrimaryContextPass, panels::draw_root_panel)
        .add_systems(
            PostUpdate,
            selection::update_viewport_selection.after(update_prop_hover),
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
