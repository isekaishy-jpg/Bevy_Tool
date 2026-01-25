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

use create::{create_new_project, create_new_world};
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

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveRegion {
    pub region_id: Option<String>,
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
    CreateWorld {
        root: PathBuf,
        request: NewWorldRequest,
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

#[derive(Debug, Clone)]
pub struct NewWorldRequest {
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
    mut active_region: ResMut<ActiveRegion>,
    mut recovery_state: ResMut<RecoveryState>,
) {
    match event.event() {
        ProjectCommand::Open { root } => match open_project(root.as_path(), &mut editor_state) {
            Ok(info) => {
                update_config(&mut config, &info);
                prefs.record_project(&info.root, info.manifest.project_name.clone());
                refresh_recovery_state(&mut recovery_state, &info.root);
                state.current = Some(info);
                set_active_region(&mut active_region, state.current.as_ref());
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
                    set_active_region(&mut active_region, state.current.as_ref());
                    state.last_error = None;
                }
                Err(err) => {
                    state.last_error = Some(format!("create project failed: {err}"));
                    warn!("create project failed: {err}");
                }
            }
        }
        ProjectCommand::CreateWorld { root, request } => {
            let Some(current) = &mut state.current else {
                state.last_error = Some("create world failed: no project open".to_string());
                warn!("create world failed: no project open");
                return;
            };
            if current.root != *root {
                state.last_error = Some("create world failed: project mismatch".to_string());
                warn!("create world failed: project mismatch");
                return;
            }
            match create_new_world(root.as_path(), &current.manifest, request) {
                Ok(world) => {
                    current.current_world_id = Some(world.manifest.world_id.clone());
                    editor_state.state.last_world_id = Some(world.manifest.world_id.clone());
                    config.world_name = world.manifest.world_name.clone();
                    current.worlds.push(world);
                    set_active_region(&mut active_region, state.current.as_ref());
                    state.last_error = None;
                }
                Err(err) => {
                    state.last_error = Some(format!("create world failed: {err}"));
                    warn!("create world failed: {err}");
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
                    set_active_region(&mut active_region, state.current.as_ref());
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
                    set_active_region(&mut active_region, state.current.as_ref());
                }
            }
        }
    }
}

fn set_active_region(active_region: &mut ActiveRegion, project: Option<&ProjectInfo>) {
    let Some(project) = project else {
        active_region.region_id = None;
        return;
    };
    let Some(world) = project.current_world() else {
        active_region.region_id = None;
        return;
    };
    active_region.region_id = world
        .manifest
        .regions
        .first()
        .map(|region| region.region_id.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::schema::{ProjectManifest, RegionBounds, RegionManifest, WorldManifest};

    fn make_project(world_id: &str, regions: Vec<RegionManifest>) -> ProjectInfo {
        let manifest = WorldManifest {
            world_id: world_id.to_string(),
            regions,
            ..Default::default()
        };

        ProjectInfo {
            root: PathBuf::from("root"),
            manifest: ProjectManifest::default(),
            worlds: vec![WorldInfo {
                root: PathBuf::from("root"),
                manifest,
                has_tiles: false,
            }],
            current_world_id: Some(world_id.to_string()),
        }
    }

    #[test]
    fn set_active_region_none_project_clears_selection() {
        let mut active = ActiveRegion {
            region_id: Some("region_0".to_string()),
        };
        set_active_region(&mut active, None);
        assert!(active.region_id.is_none());
    }

    #[test]
    fn set_active_region_empty_regions_clears_selection() {
        let project = make_project("world_0", Vec::new());
        let mut active = ActiveRegion::default();
        set_active_region(&mut active, Some(&project));
        assert!(active.region_id.is_none());
    }

    #[test]
    fn set_active_region_selects_first_region() {
        let project = make_project(
            "world_0",
            vec![
                RegionManifest {
                    region_id: "region_a".to_string(),
                    name: "Region A".to_string(),
                    bounds: RegionBounds::new(0, 0, 1, 1),
                },
                RegionManifest {
                    region_id: "region_b".to_string(),
                    name: "Region B".to_string(),
                    bounds: RegionBounds::new(0, 0, 1, 1),
                },
            ],
        );
        let mut active = ActiveRegion::default();
        set_active_region(&mut active, Some(&project));
        assert_eq!(active.region_id.as_deref(), Some("region_a"));
    }
}
