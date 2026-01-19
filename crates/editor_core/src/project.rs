use std::fs;
use std::path::{Path, PathBuf};

use bevy::log::warn;
use bevy::prelude::*;
use world::schema::{ProjectManifest, RegionBounds, RegionManifest, WorldManifest, WorldSpec};
use world::storage::{
    create_project, create_region, create_world, project_layout, read_project_manifest,
    read_world_manifest, region_tiles_dir, world_layout, write_project_manifest,
    write_world_manifest, ProjectLayout,
};

use crate::autosave::{clear_recovery_state, refresh_recovery_state, RecoveryState};
use crate::editor_state::{load_project_editor_state, ProjectEditorStateResource};
use crate::prefs::EditorPrefs;
use crate::EditorConfig;

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

fn open_project(
    root: &Path,
    editor_state: &mut ProjectEditorStateResource,
) -> anyhow::Result<ProjectInfo> {
    let manifest = read_project_manifest(root)?;
    let layout = project_layout(root, &manifest);
    let worlds = load_worlds(&layout)?;

    let loaded_state = load_project_editor_state(root).unwrap_or_default();
    editor_state.root = Some(root.to_path_buf());
    editor_state.state = loaded_state;

    let current_world_id = choose_world_id(&worlds, editor_state.state.last_world_id.as_deref());
    editor_state.state.last_world_id = current_world_id.clone();

    Ok(ProjectInfo {
        root: root.to_path_buf(),
        manifest,
        worlds,
        current_world_id,
    })
}

fn create_new_project(
    root: &Path,
    request: &NewProjectRequest,
    editor_state: &mut ProjectEditorStateResource,
) -> anyhow::Result<ProjectInfo> {
    if !request.region_bounds.is_valid() {
        return Err(anyhow::anyhow!("region bounds are invalid"));
    }

    let project_name = if request.project_name.trim().is_empty() {
        "NewProject".to_string()
    } else {
        request.project_name.trim().to_string()
    };
    let world_name = if request.world_name.trim().is_empty() {
        "NewWorld".to_string()
    } else {
        request.world_name.trim().to_string()
    };
    let region_id = if request.region_id.trim().is_empty() {
        "r000".to_string()
    } else {
        request.region_id.trim().to_string()
    };
    let region_name = if request.region_name.trim().is_empty() {
        "Region 0".to_string()
    } else {
        request.region_name.trim().to_string()
    };

    let project_manifest = ProjectManifest {
        project_id: new_uuid(),
        project_name,
        created_unix_ms: now_unix_ms(),
        ..ProjectManifest::default()
    };
    let world_manifest = WorldManifest {
        world_id: new_uuid(),
        world_name,
        world_spec: request.world_spec,
        regions: vec![RegionManifest {
            region_id,
            name: region_name,
            bounds: request.region_bounds,
        }],
        ..WorldManifest::default()
    };

    let layout = create_project(root, &project_manifest)?;
    let world_layout = create_world(&layout, &world_manifest)?;
    let world_info = WorldInfo {
        root: world_layout.world_root.clone(),
        manifest: world_manifest.clone(),
        has_tiles: world_has_tiles(&layout, &world_manifest),
    };

    editor_state.root = Some(root.to_path_buf());
    editor_state.state = Default::default();
    editor_state.state.last_world_id = Some(world_manifest.world_id.clone());

    Ok(ProjectInfo {
        root: root.to_path_buf(),
        manifest: project_manifest.clone(),
        worlds: vec![world_info],
        current_world_id: Some(world_manifest.world_id.clone()),
    })
}

fn load_worlds(layout: &ProjectLayout) -> anyhow::Result<Vec<WorldInfo>> {
    let mut worlds = Vec::new();
    let Ok(entries) = fs::read_dir(&layout.worlds_dir) else {
        return Ok(worlds);
    };
    for entry in entries.flatten() {
        let world_root = entry.path();
        if !world_root.is_dir() {
            continue;
        }
        let manifest = read_world_manifest(&world_root)?;
        worlds.push(WorldInfo {
            root: world_root.clone(),
            has_tiles: world_has_tiles(layout, &manifest),
            manifest,
        });
    }
    Ok(worlds)
}

fn world_has_tiles(layout: &ProjectLayout, manifest: &WorldManifest) -> bool {
    let world_layout = world_layout(layout, &manifest.world_id);
    for region in &manifest.regions {
        let tiles_dir = region_tiles_dir(&world_layout, &region.region_id);
        let Ok(entries) = fs::read_dir(&tiles_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("tile") {
                return true;
            }
        }
    }
    false
}

fn ensure_world_regions(root: &Path, project: &ProjectManifest, world: &WorldManifest) {
    let layout = project_layout(root, project);
    let world_layout = world_layout(&layout, &world.world_id);
    for region in &world.regions {
        if let Err(err) = create_region(&world_layout, region) {
            warn!("failed to create region {}: {err}", region.region_id);
        }
    }
}

fn choose_world_id(worlds: &[WorldInfo], preferred: Option<&str>) -> Option<String> {
    if let Some(preferred) = preferred {
        if worlds
            .iter()
            .any(|world| world.manifest.world_id == preferred)
        {
            return Some(preferred.to_string());
        }
    }
    worlds.first().map(|world| world.manifest.world_id.clone())
}

fn update_config(config: &mut EditorConfig, info: &ProjectInfo) {
    config.project_name = info.manifest.project_name.clone();
    config.world_name = info
        .current_world()
        .map(|world| world.manifest.world_name.clone())
        .unwrap_or_else(|| "Untitled".to_string());
}

fn now_unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
