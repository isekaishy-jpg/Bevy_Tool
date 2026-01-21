use bevy::log::warn;
use bevy::prelude::*;
use std::path::PathBuf;
use world::schema::{ProjectManifest, RegionBounds, WorldManifest, WorldSpec};
use world::storage::{project_layout, write_project_manifest, write_world_manifest};

use crate::autosave::{clear_recovery_state, refresh_recovery_state, RecoveryState};
use crate::editor_state::ProjectEditorStateResource;
use crate::prefs::EditorPrefs;
use crate::EditorConfig;

mod create;
mod helpers;
mod open;

use create::create_new_project;
use helpers::{ensure_world_regions, update_config, world_has_tiles};
use open::open_project;

#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub root: PathBuf,
    pub manifest: WorldManifest,
    pub has_tiles: bool,
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub worlds: Vec<WorldInfo>,
    pub current_world_id: Option<String>,
}

impl ProjectInfo {
    pub fn current_world(&self) -> Option<&WorldInfo> {
        let id = self.current_world_id.as_deref()?;
        self.worlds
            .iter()
            .find(|world| world.manifest.world_id == id)
    }
}

#[derive(Resource, Default)]
pub struct ProjectState {
    pub current: Option<ProjectInfo>,
    pub last_error: Option<String>,
}

#[derive(Event)]
pub enum ProjectCommand {
    Open {
        root: PathBuf,
    },
    Create {
        root: PathBuf,
        request: NewProjectRequest,
    },
    UpdateProjectManifest {
        root: PathBuf,
        manifest: ProjectManifest,
    },
    UpdateWorldManifest {
        root: PathBuf,
        manifest: WorldManifest,
    },
    SetCurrentWorld {
        world_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct NewProjectRequest {
    pub project_name: String,
    pub world_name: String,
    pub region_id: String,
    pub region_name: String,
    pub region_bounds: RegionBounds,
    pub world_spec: WorldSpec,
}

pub fn apply_project_commands(
    event: On<ProjectCommand>,
    mut state: ResMut<ProjectState>,
    mut config: ResMut<EditorConfig>,
    mut prefs: ResMut<EditorPrefs>,
    mut editor_state: ResMut<ProjectEditorStateResource>,
    mut recovery_state: ResMut<RecoveryState>,
) {
    match event.event() {
        ProjectCommand::Open { root } => match open_project(root.as_path(), &mut editor_state) {
            Ok(info) => {
                update_config(&mut config, &info);
                prefs.record_project(&info.root, info.manifest.project_name.clone());
                refresh_recovery_state(&mut recovery_state, &info.root);
                state.current = Some(info);
                state.last_error = None;
            }
            Err(err) => {
                state.last_error = Some(format!("open project failed: {err}"));
                warn!("open project failed: {err}");
            }
        },
        ProjectCommand::Create { root, request } => {
            match create_new_project(root.as_path(), request, &mut editor_state) {
                Ok(info) => {
                    update_config(&mut config, &info);
                    prefs.record_project(&info.root, info.manifest.project_name.clone());
                    clear_recovery_state(&mut recovery_state, &info.root);
                    state.current = Some(info);
                    state.last_error = None;
                }
                Err(err) => {
                    state.last_error = Some(format!("create project failed: {err}"));
                    warn!("create project failed: {err}");
                }
            }
        }
        ProjectCommand::UpdateProjectManifest { root, manifest } => {
            if let Err(err) = write_project_manifest(root.as_path(), manifest) {
                state.last_error = Some(format!("save project failed: {err}"));
                warn!("save project failed: {err}");
                return;
            }
            if let Some(current) = &mut state.current {
                if current.root == *root {
                    current.manifest = manifest.clone();
                    config.project_name = manifest.project_name.clone();
                    prefs.update_display_name(root.as_path(), &manifest.project_name);
                    state.last_error = None;
                }
            }
        }
        ProjectCommand::UpdateWorldManifest { root, manifest } => {
            let Some(current) = &mut state.current else {
                state.last_error = Some("save world failed: no project open".to_string());
                warn!("save world failed: no project open");
                return;
            };

            if current.root != *root {
                state.last_error = Some("save world failed: project mismatch".to_string());
                warn!("save world failed: project mismatch");
                return;
            }

            let layout = project_layout(root.as_path(), &current.manifest);
            let world_root = layout.worlds_dir.join(&manifest.world_id);
            if let Err(err) = write_world_manifest(&world_root, manifest) {
                state.last_error = Some(format!("save world failed: {err}"));
                warn!("save world failed: {err}");
                return;
            }

            if let Some(world) = current
                .worlds
                .iter_mut()
                .find(|world| world.manifest.world_id == manifest.world_id)
            {
                ensure_world_regions(root, &current.manifest, manifest);
                world.has_tiles = world_has_tiles(&layout, manifest);
                world.manifest = manifest.clone();
                if current.current_world_id.as_deref() == Some(manifest.world_id.as_str()) {
                    config.world_name = manifest.world_name.clone();
                }
                state.last_error = None;
            }
        }
        ProjectCommand::SetCurrentWorld { world_id } => {
            if let Some(current) = &mut state.current {
                if current
                    .worlds
                    .iter()
                    .any(|world| world.manifest.world_id == *world_id)
                {
                    current.current_world_id = Some(world_id.clone());
                    editor_state.state.last_world_id = Some(world_id.clone());
                    if let Some(world) = current.current_world() {
                        config.world_name = world.manifest.world_name.clone();
                    }
                }
            }
        }
    }
}
