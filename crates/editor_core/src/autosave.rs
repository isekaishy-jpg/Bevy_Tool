use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use bevy::log::warn;
use bevy::prelude::{Res, ResMut, Resource, Time};

use crate::editor_state::{save_project_editor_state, ProjectEditorStateResource};
use crate::project::ProjectInfo;
use crate::project::ProjectState;

use world::storage::{
    write_project_manifest, write_world_manifest, PROJECT_MANIFEST_FILE, WORLD_MANIFEST_FILE,
};

const DEFAULT_AUTOSAVE_INTERVAL_SECS: u64 = 60;
const BACKUP_RETENTION: usize = 5;
const EDITOR_DIR_NAME: &str = ".editor";
const BACKUP_DIR_NAME: &str = "backups";
const EDITOR_STATE_FILE: &str = "editor_state.toml";

#[derive(Resource, Debug, Clone)]
pub struct AutosaveSettings {
    pub interval: Duration,
}

impl Default for AutosaveSettings {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_AUTOSAVE_INTERVAL_SECS),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct AutosaveState {
    pub last_autosave_secs: f64,
}

#[derive(Clone, Debug)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub timestamp: u64,
}

#[derive(Resource, Debug, Default)]
pub struct RecoveryState {
    pub project_root: Option<PathBuf>,
    pub pending_backup: Option<BackupInfo>,
    pub dismissed: bool,
}

pub fn autosave_system(
    time: Res<Time>,
    project_state: Res<ProjectState>,
    editor_state: Res<ProjectEditorStateResource>,
    settings: Res<AutosaveSettings>,
    mut autosave_state: ResMut<AutosaveState>,
) {
    if !editor_state.state.autosave_enabled {
        return;
    }
    let Some(project) = &project_state.current else {
        return;
    };
    let elapsed = time.elapsed_secs_f64();
    if elapsed - autosave_state.last_autosave_secs < settings.interval.as_secs_f64() {
        return;
    }

    if let Err(err) = autosave_project(project, &editor_state) {
        warn!("autosave failed: {err}");
        return;
    }
    autosave_state.last_autosave_secs = elapsed;
    bevy::log::info!("autosave complete");
}

pub fn refresh_recovery_state(recovery: &mut RecoveryState, project_root: &Path) {
    recovery.project_root = Some(project_root.to_path_buf());
    recovery.pending_backup = find_latest_backup(project_root);
    recovery.dismissed = false;
}

pub fn clear_recovery_state(recovery: &mut RecoveryState, project_root: &Path) {
    recovery.project_root = Some(project_root.to_path_buf());
    recovery.pending_backup = None;
    recovery.dismissed = true;
}

pub fn autosave_project(
    project: &ProjectInfo,
    editor_state: &ProjectEditorStateResource,
) -> anyhow::Result<()> {
    write_project_manifest(&project.root, &project.manifest)
        .with_context(|| format!("write project manifest {:?}", project.root))?;

    let worlds_dir = project.root.join(&project.manifest.worlds_dir);
    for world in &project.worlds {
        let world_root = worlds_dir.join(&world.manifest.world_id);
        write_world_manifest(&world_root, &world.manifest)
            .with_context(|| format!("write world manifest {:?}", world_root))?;
    }

    if let Some(root) = &editor_state.root {
        save_project_editor_state(root, &editor_state.state)?;
    }

    write_backup_snapshot(project, editor_state)?;
    Ok(())
}

pub fn restore_backup(project_root: &Path, backup_dir: &Path) -> anyhow::Result<()> {
    let project_manifest_src = backup_dir.join(PROJECT_MANIFEST_FILE);
    let project_manifest_dst = project_root.join(PROJECT_MANIFEST_FILE);
    if project_manifest_src.exists() {
        fs::copy(&project_manifest_src, &project_manifest_dst)
            .with_context(|| format!("restore project manifest {:?}", project_manifest_dst))?;
    }

    let editor_state_src = backup_dir.join(EDITOR_STATE_FILE);
    if editor_state_src.exists() {
        let editor_state_dst = project_root.join(EDITOR_DIR_NAME).join(EDITOR_STATE_FILE);
        if let Some(parent) = editor_state_dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create editor state dir {:?}", parent))?;
        }
        fs::copy(&editor_state_src, &editor_state_dst)
            .with_context(|| format!("restore editor state {:?}", editor_state_dst))?;
    }

    let worlds_src = backup_dir.join("worlds");
    if worlds_src.exists() {
        let world_dirs = fs::read_dir(&worlds_src)
            .with_context(|| format!("read backup worlds {:?}", worlds_src))?;
        for entry in world_dirs.flatten() {
            let world_root = entry.path();
            if !world_root.is_dir() {
                continue;
            }
            let world_id = match world_root.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            let src = world_root.join(WORLD_MANIFEST_FILE);
            if !src.exists() {
                continue;
            }
            let dst = project_root
                .join("worlds")
                .join(&world_id)
                .join(WORLD_MANIFEST_FILE);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create world dir {:?}", parent))?;
            }
            fs::copy(&src, &dst).with_context(|| format!("restore world {:?}", dst))?;
        }
    }

    Ok(())
}

fn write_backup_snapshot(
    project: &ProjectInfo,
    editor_state: &ProjectEditorStateResource,
) -> anyhow::Result<()> {
    let backups_dir = backup_dir(&project.root);
    let timestamp = now_unix_ms();
    let backup_root = backups_dir.join(timestamp.to_string());
    let world_backup_dir = backup_root.join("worlds");

    fs::create_dir_all(&world_backup_dir)
        .with_context(|| format!("create backup dir {:?}", world_backup_dir))?;

    let project_manifest_src = project.root.join(PROJECT_MANIFEST_FILE);
    let project_manifest_dst = backup_root.join(PROJECT_MANIFEST_FILE);
    if project_manifest_src.exists() {
        fs::copy(&project_manifest_src, &project_manifest_dst)
            .with_context(|| format!("backup project manifest {:?}", project_manifest_dst))?;
    }

    if let Some(root) = &editor_state.root {
        let editor_state_src = root.join(EDITOR_DIR_NAME).join(EDITOR_STATE_FILE);
        if editor_state_src.exists() {
            let editor_state_dst = backup_root.join(EDITOR_STATE_FILE);
            fs::copy(&editor_state_src, &editor_state_dst)
                .with_context(|| format!("backup editor state {:?}", editor_state_dst))?;
        }
    }

    let worlds_dir = project.root.join(&project.manifest.worlds_dir);
    for world in &project.worlds {
        let world_src = worlds_dir
            .join(&world.manifest.world_id)
            .join(WORLD_MANIFEST_FILE);
        if !world_src.exists() {
            continue;
        }
        let world_dst_dir = world_backup_dir.join(&world.manifest.world_id);
        fs::create_dir_all(&world_dst_dir)
            .with_context(|| format!("create world backup dir {:?}", world_dst_dir))?;
        let world_dst = world_dst_dir.join(WORLD_MANIFEST_FILE);
        fs::copy(&world_src, &world_dst)
            .with_context(|| format!("backup world manifest {:?}", world_dst))?;
    }

    rotate_backups(&backups_dir)?;
    Ok(())
}

fn rotate_backups(backups_dir: &Path) -> anyhow::Result<()> {
    let Ok(entries) = fs::read_dir(backups_dir) else {
        return Ok(());
    };
    let mut timestamps = Vec::new();
    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if let Ok(timestamp) = name.parse::<u64>() {
                timestamps.push((timestamp, entry.path()));
            }
        }
    }
    timestamps.sort_by_key(|(timestamp, _)| *timestamp);
    if timestamps.len() <= BACKUP_RETENTION {
        return Ok(());
    }
    let remove_count = timestamps.len() - BACKUP_RETENTION;
    for (_, path) in timestamps.into_iter().take(remove_count) {
        let _ = fs::remove_dir_all(&path);
    }
    Ok(())
}

fn find_latest_backup(project_root: &Path) -> Option<BackupInfo> {
    let backups_dir = backup_dir(project_root);
    let Ok(entries) = fs::read_dir(&backups_dir) else {
        return None;
    };
    let mut latest = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Ok(timestamp) = name.parse::<u64>() else {
            continue;
        };
        if let Some((current, _)) = latest {
            if timestamp <= current {
                continue;
            }
        }
        latest = Some((timestamp, entry.path()));
    }
    latest.map(|(timestamp, path)| BackupInfo { path, timestamp })
}

fn backup_dir(project_root: &Path) -> PathBuf {
    project_root.join(EDITOR_DIR_NAME).join(BACKUP_DIR_NAME)
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
