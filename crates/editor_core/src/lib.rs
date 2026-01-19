//! Editor application assembly (plugins, commands, undo/redo).

use bevy::log::warn;
use bevy::prelude::*;

pub mod command_registry;
pub mod commands;
pub mod prefs;
pub mod project;

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
        let prefs = prefs::load_editor_prefs().unwrap_or_else(|err| {
            warn!("failed to load editor prefs: {err}");
            prefs::EditorPrefs::default()
        });

        app.init_resource::<EditorConfig>();
        app.init_resource::<project::ProjectState>();
        app.init_resource::<commands::CommandStack>();
        app.init_resource::<command_registry::OverlayState>();
        app.init_resource::<command_registry::FocusSelectionRequest>();
        app.insert_resource(command_registry::CommandRegistry::new_default());
        app.insert_resource(prefs);
        app.add_observer(project::apply_project_commands);
        app.add_observer(command_registry::handle_command_invoked);
        app.add_systems(Startup, command_registry::validate_command_registry);
        app.add_systems(Update, prefs::save_prefs_on_change);
    }
}
