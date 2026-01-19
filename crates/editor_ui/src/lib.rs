//! Egui panels, docking, widgets.

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub mod panels;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<panels::DockLayout>()
            .add_systems(EguiPrimaryContextPass, panels::draw_root_panel);
    }
}
