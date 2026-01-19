//! Command registry and execution.

use bevy::log::{info, warn};
use bevy::prelude::*;
use world::storage::{write_project_manifest, write_world_manifest};

use crate::commands::CommandStack;
use crate::project::ProjectState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    File,
    Edit,
    View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    Save,
    SaveAllDirty,
    Undo,
    Redo,
    FocusSelection,
    ToggleOverlays,
}

impl CommandId {
    pub const fn as_str(self) -> &'static str {
        match self {
            CommandId::Save => "save",
            CommandId::SaveAllDirty => "save_all_dirty",
            CommandId::Undo => "undo",
            CommandId::Redo => "redo",
            CommandId::FocusSelection => "focus_selection",
            CommandId::ToggleOverlays => "toggle_overlays",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandDef {
    pub id: CommandId,
    pub name: &'static str,
    pub description: &'static str,
    pub category: CommandCategory,
    pub hotkey: Option<&'static str>,
}

#[derive(Resource, Debug, Default)]
pub struct CommandRegistry {
    pub commands: Vec<CommandDef>,
}

impl CommandRegistry {
    pub fn new_default() -> Self {
        Self {
            commands: vec![
                CommandDef {
                    id: CommandId::Save,
                    name: "Save",
                    description: "Save the current project",
                    category: CommandCategory::File,
                    hotkey: Some("Ctrl+S"),
                },
                CommandDef {
                    id: CommandId::SaveAllDirty,
                    name: "Save All",
                    description: "Save all dirty assets",
                    category: CommandCategory::File,
                    hotkey: Some("Ctrl+Shift+S"),
                },
                CommandDef {
                    id: CommandId::Undo,
                    name: "Undo",
                    description: "Undo the last action",
                    category: CommandCategory::Edit,
                    hotkey: Some("Ctrl+Z"),
                },
                CommandDef {
                    id: CommandId::Redo,
                    name: "Redo",
                    description: "Redo the last undone action",
                    category: CommandCategory::Edit,
                    hotkey: Some("Ctrl+Y"),
                },
                CommandDef {
                    id: CommandId::FocusSelection,
                    name: "Focus Selection",
                    description: "Frame the current selection",
                    category: CommandCategory::View,
                    hotkey: Some("F"),
                },
                CommandDef {
                    id: CommandId::ToggleOverlays,
                    name: "Toggle Overlays",
                    description: "Show/hide editor overlays",
                    category: CommandCategory::View,
                    hotkey: Some("O"),
                },
            ],
        }
    }

    pub fn find(&self, id: CommandId) -> Option<&CommandDef> {
        self.commands.iter().find(|cmd| cmd.id == id)
    }
}

#[derive(Event)]
pub struct CommandInvoked {
    pub id: CommandId,
}

#[derive(Resource, Debug)]
pub struct OverlayState {
    pub show_overlays: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            show_overlays: true,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct FocusSelectionRequest {
    pub requested: bool,
}

pub fn validate_command_registry(registry: Res<CommandRegistry>) {
    use std::collections::HashSet;

    let mut ids = HashSet::new();
    let mut hotkeys = HashSet::new();
    for command in &registry.commands {
        if !ids.insert(command.id) {
            warn!("duplicate command id: {}", command.id.as_str());
        }
        if let Some(hotkey) = command.hotkey {
            if !hotkeys.insert(hotkey) {
                warn!("duplicate hotkey binding: {}", hotkey);
            }
        }
    }
}

pub fn handle_command_invoked(
    event: On<CommandInvoked>,
    mut project_state: ResMut<ProjectState>,
    mut command_stack: ResMut<CommandStack>,
    mut overlays: ResMut<OverlayState>,
    mut focus_request: ResMut<FocusSelectionRequest>,
) {
    match event.event().id {
        CommandId::Save | CommandId::SaveAllDirty => {
            let Some(project) = &project_state.current else {
                project_state.last_error = Some("save failed: no project open".to_string());
                warn!("save failed: no project open");
                return;
            };

            if let Err(err) = write_project_manifest(&project.root, &project.manifest) {
                project_state.last_error = Some(format!("save failed: {err}"));
                warn!("save failed: {err}");
                return;
            }

            let worlds_dir = project.root.join(&project.manifest.worlds_dir);
            let save_all = matches!(event.event().id, CommandId::SaveAllDirty);
            let worlds = if save_all {
                project.worlds.iter().collect::<Vec<_>>()
            } else {
                project.current_world().into_iter().collect::<Vec<_>>()
            };

            for world in worlds {
                let world_root = worlds_dir.join(&world.manifest.world_id);
                if let Err(err) = write_world_manifest(&world_root, &world.manifest) {
                    project_state.last_error = Some(format!("save failed: {err}"));
                    warn!("save failed: {err}");
                    return;
                }
            }

            info!("saved project {:?}", project.root);
        }
        CommandId::Undo => {
            if command_stack.undo().is_none() {
                info!("undo requested but stack is empty");
            }
        }
        CommandId::Redo => {
            if command_stack.redo().is_none() {
                info!("redo requested but stack is empty");
            }
        }
        CommandId::FocusSelection => {
            focus_request.requested = true;
            info!("focus selection requested");
        }
        CommandId::ToggleOverlays => {
            overlays.show_overlays = !overlays.show_overlays;
            info!("overlays enabled: {}", overlays.show_overlays);
        }
    }
}
