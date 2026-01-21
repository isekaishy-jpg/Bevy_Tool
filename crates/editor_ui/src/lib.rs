//! Egui panels, docking, widgets.

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};

pub mod panels;

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
        .add_systems(Startup, setup_ui_camera)
        .add_systems(EguiPrimaryContextPass, panels::draw_root_panel);
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
