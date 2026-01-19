//! Editor application assembly (plugins, commands, undo/redo).

use bevy::prelude::*;

pub mod commands;

#[derive(Resource)]
pub struct EditorConfig {
    pub project_name: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            project_name: "Untitled".to_string(),
        }
    }
}

pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorConfig>();
    }
}
