use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui;
use editor_core::prefs::EditorPrefs;
use editor_core::project::{ProjectCommand, ProjectManifest, ProjectState};

#[derive(Resource)]
pub struct ProjectPanelState {
    pub new_world_name: String,
    pub new_project_path: String,
    pub open_project_path: String,
    pub project_key: Option<String>,
    pub world_name_edit: String,
    pub tile_size_meters: f32,
    pub chunk_resolution: u16,
    pub heightfield_resolution: u16,
    pub weightmap_resolution: u16,
    pub liquids_resolution: u16,
    pub pending_commands: Vec<ProjectCommand>,
}

impl Default for ProjectPanelState {
    fn default() -> Self {
        let manifest = ProjectManifest::default();
        Self {
            new_world_name: manifest.world_name.clone(),
            new_project_path: String::new(),
            open_project_path: String::new(),
            project_key: None,
            world_name_edit: String::new(),
            tile_size_meters: manifest.tile_size_meters,
            chunk_resolution: manifest.chunk_resolution,
            heightfield_resolution: manifest.heightfield_resolution,
            weightmap_resolution: manifest.weightmap_resolution,
            liquids_resolution: manifest.liquids_resolution,
            pending_commands: Vec::new(),
        }
    }
}

pub fn draw_project_panel(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    project_state: &ProjectState,
    prefs: &mut EditorPrefs,
) {
    ui.heading("Project");
    ui.separator();

    if let Some(error) = &project_state.last_error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.separator();
    }

    if let Some(info) = &project_state.current {
        sync_from_project(state, info);

        ui.label(format!("Root: {}", info.root.display()));
        ui.separator();

        ui.heading("Settings");
        ui.horizontal(|ui| {
            ui.label("World name");
            ui.text_edit_singleline(&mut state.world_name_edit);
        });

        let spec_locked = info.has_tiles;
        if spec_locked {
            ui.label("World spec locked because tiles exist on disk.");
        }

        ui.add_enabled_ui(!spec_locked, |ui| {
            ui.horizontal(|ui| {
                ui.label("Tile size (m)");
                ui.add(egui::DragValue::new(&mut state.tile_size_meters).speed(1.0));
            });
            ui.horizontal(|ui| {
                ui.label("Chunks per tile");
                ui.add(egui::DragValue::new(&mut state.chunk_resolution).range(1..=64));
            });
            ui.horizontal(|ui| {
                ui.label("Heightfield resolution");
                ui.add(egui::DragValue::new(&mut state.heightfield_resolution).range(2..=4096));
            });
            ui.horizontal(|ui| {
                ui.label("Weightmap resolution");
                ui.add(egui::DragValue::new(&mut state.weightmap_resolution).range(2..=4096));
            });
            ui.horizontal(|ui| {
                ui.label("Liquids resolution");
                ui.add(egui::DragValue::new(&mut state.liquids_resolution).range(2..=4096));
            });
        });

        let dirty = is_manifest_dirty(state, &info.manifest);
        if ui
            .add_enabled(dirty, egui::Button::new("Save Settings"))
            .clicked()
        {
            let updated = manifest_from_state(state, &info.manifest);
            state.pending_commands.push(ProjectCommand::UpdateManifest {
                root: info.root.clone(),
                manifest: updated,
            });
        }

        ui.separator();
    } else {
        ui.label("No project open.");
        ui.separator();
    }

    ui.collapsing("New Project", |ui| {
        ui.horizontal(|ui| {
            ui.label("World name");
            ui.text_edit_singleline(&mut state.new_world_name);
        });

        ui.horizontal(|ui| {
            ui.label("Project folder");
            ui.text_edit_singleline(&mut state.new_project_path);
            if ui.button("Pick...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.new_project_path = path.display().to_string();
                }
            }
        });

        let can_create = !state.new_project_path.trim().is_empty();
        if ui
            .add_enabled(can_create, egui::Button::new("Create Project"))
            .clicked()
        {
            let root = PathBuf::from(state.new_project_path.trim());
            let manifest = new_manifest_from_state(state);
            state
                .pending_commands
                .push(ProjectCommand::Create { root, manifest });
        }
    });

    ui.collapsing("Open Project", |ui| {
        ui.horizontal(|ui| {
            ui.label("Project folder");
            ui.text_edit_singleline(&mut state.open_project_path);
            if ui.button("Pick...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.open_project_path = path.display().to_string();
                }
            }
        });

        let can_open = !state.open_project_path.trim().is_empty();
        if ui
            .add_enabled(can_open, egui::Button::new("Open Project"))
            .clicked()
        {
            let root = PathBuf::from(state.open_project_path.trim());
            state.pending_commands.push(ProjectCommand::Open { root });
        }
    });

    ui.separator();
    ui.heading("Recent Projects");

    if prefs.recent_projects.is_empty() {
        ui.label("No recent projects yet.");
    } else {
        let mut remove_index = None;
        for (index, recent) in prefs.recent_projects.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(&recent.display_name);
                ui.separator();
                ui.label(&recent.path);
                let exists = Path::new(&recent.path).exists();
                if !exists {
                    ui.colored_label(egui::Color32::LIGHT_RED, "Missing");
                }
                if ui.add_enabled(exists, egui::Button::new("Open")).clicked() {
                    state.pending_commands.push(ProjectCommand::Open {
                        root: PathBuf::from(&recent.path),
                    });
                }
                if ui.button("Remove").clicked() {
                    remove_index = Some(index);
                }
            });
        }

        if let Some(index) = remove_index {
            let path = prefs.recent_projects[index].path.clone();
            prefs.remove_recent(Path::new(&path));
        }
    }
}

fn sync_from_project(state: &mut ProjectPanelState, info: &editor_core::project::ProjectInfo) {
    let key = info.root.to_string_lossy().to_string();
    if state.project_key.as_deref() == Some(&key) {
        return;
    }

    state.project_key = Some(key);
    state.world_name_edit = info.manifest.world_name.clone();
    state.tile_size_meters = info.manifest.tile_size_meters;
    state.chunk_resolution = info.manifest.chunk_resolution;
    state.heightfield_resolution = info.manifest.heightfield_resolution;
    state.weightmap_resolution = info.manifest.weightmap_resolution;
    state.liquids_resolution = info.manifest.liquids_resolution;
}

fn is_manifest_dirty(state: &ProjectPanelState, manifest: &ProjectManifest) -> bool {
    state.world_name_edit.trim() != manifest.world_name
        || state.tile_size_meters != manifest.tile_size_meters
        || state.chunk_resolution != manifest.chunk_resolution
        || state.heightfield_resolution != manifest.heightfield_resolution
        || state.weightmap_resolution != manifest.weightmap_resolution
        || state.liquids_resolution != manifest.liquids_resolution
}

fn manifest_from_state(state: &ProjectPanelState, manifest: &ProjectManifest) -> ProjectManifest {
    let mut updated = manifest.clone();
    let world_name = state.world_name_edit.trim();
    updated.world_name = if world_name.is_empty() {
        manifest.world_name.clone()
    } else {
        world_name.to_string()
    };
    updated.tile_size_meters = state.tile_size_meters;
    updated.chunk_resolution = state.chunk_resolution;
    updated.heightfield_resolution = state.heightfield_resolution;
    updated.weightmap_resolution = state.weightmap_resolution;
    updated.liquids_resolution = state.liquids_resolution;
    updated
}

fn new_manifest_from_state(state: &ProjectPanelState) -> ProjectManifest {
    let mut manifest = ProjectManifest::default();
    let world_name = state.new_world_name.trim();
    manifest.world_name = if world_name.is_empty() {
        "NewWorld".to_string()
    } else {
        world_name.to_string()
    };
    manifest.tile_size_meters = state.tile_size_meters;
    manifest.chunk_resolution = state.chunk_resolution;
    manifest.heightfield_resolution = state.heightfield_resolution;
    manifest.weightmap_resolution = state.weightmap_resolution;
    manifest.liquids_resolution = state.liquids_resolution;
    manifest
}
