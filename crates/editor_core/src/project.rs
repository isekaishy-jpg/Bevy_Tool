use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use bevy::log::warn;
use bevy::prelude::*;
use world::storage::{create_project, read_manifest};

use crate::prefs::EditorPrefs;
use crate::EditorConfig;

pub use world::schema::ProjectManifest;

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub has_tiles: bool,
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
        manifest: ProjectManifest,
    },
    UpdateManifest {
        root: PathBuf,
        manifest: ProjectManifest,
    },
}

pub fn apply_project_commands(
    event: On<ProjectCommand>,
    mut state: ResMut<ProjectState>,
    mut config: ResMut<EditorConfig>,
    mut prefs: ResMut<EditorPrefs>,
) {
    match event.event() {
        ProjectCommand::Open { root } => match open_project(root.as_path()) {
            Ok(info) => {
                config.project_name = info.manifest.world_name.clone();
                prefs.record_project(&info.root, info.manifest.world_name.clone());
                state.current = Some(info);
                state.last_error = None;
            }
            Err(err) => {
                state.last_error = Some(format!("open project failed: {err}"));
                warn!("open project failed: {err}");
            }
        },
        ProjectCommand::Create { root, manifest } => {
            match create_new_project(root.as_path(), manifest) {
                Ok(info) => {
                    config.project_name = info.manifest.world_name.clone();
                    prefs.record_project(&info.root, info.manifest.world_name.clone());
                    state.current = Some(info);
                    state.last_error = None;
                }
                Err(err) => {
                    state.last_error = Some(format!("create project failed: {err}"));
                    warn!("create project failed: {err}");
                }
            }
        }
        ProjectCommand::UpdateManifest { root, manifest } => {
            if let Err(err) = write_manifest_to_disk(root.as_path(), manifest) {
                state.last_error = Some(format!("save settings failed: {err}"));
                warn!("save settings failed: {err}");
                return;
            }
            if let Some(current) = &mut state.current {
                if current.root == *root {
                    current.manifest = manifest.clone();
                    config.project_name = manifest.world_name.clone();
                    prefs.update_display_name(root.as_path(), &manifest.world_name);
                    state.last_error = None;
                }
            }
        }
    }
}

fn open_project(root: &Path) -> anyhow::Result<ProjectInfo> {
    let manifest = read_manifest(root)?;
    let has_tiles = project_has_tiles(root);
    Ok(ProjectInfo {
        root: root.to_path_buf(),
        manifest,
        has_tiles,
    })
}

fn create_new_project(root: &Path, manifest: &ProjectManifest) -> anyhow::Result<ProjectInfo> {
    create_project(root, manifest)?;
    Ok(ProjectInfo {
        root: root.to_path_buf(),
        manifest: manifest.clone(),
        has_tiles: false,
    })
}

fn project_has_tiles(root: &Path) -> bool {
    let tiles_dir = root.join("tiles");
    let Ok(regions) = fs::read_dir(&tiles_dir) else {
        return false;
    };
    for region in regions.flatten() {
        let region_path = region.path();
        if !region_path.is_dir() {
            continue;
        }
        if region_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "_quarantine")
        {
            continue;
        }
        let Ok(entries) = fs::read_dir(&region_path) else {
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

fn write_manifest_to_disk(root: &Path, manifest: &ProjectManifest) -> anyhow::Result<()> {
    world::storage::write_manifest(root, manifest)
        .with_context(|| format!("write manifest {:?}", root))
}
