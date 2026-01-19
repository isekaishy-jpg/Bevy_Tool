use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use bevy::log::warn;
use bevy::prelude::{DetectChanges, Res, Resource};
use serde::{Deserialize, Serialize};

const EDITOR_STATE_FILE: &str = "editor_state.toml";
const EDITOR_DIR_NAME: &str = ".editor";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectEditorState {
    pub dock_layout: Option<String>,
    pub last_world_id: Option<String>,
    pub autosave_enabled: bool,
}

impl Default for ProjectEditorState {
    fn default() -> Self {
        Self {
            dock_layout: None,
            last_world_id: None,
            autosave_enabled: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct ProjectEditorStateResource {
    pub root: Option<PathBuf>,
    pub state: ProjectEditorState,
}

pub fn load_project_editor_state(project_root: &Path) -> anyhow::Result<ProjectEditorState> {
    let path = editor_state_path(project_root);
    if !path.exists() {
        return Ok(ProjectEditorState::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("read editor state {:?}", path))?;
    let state = toml::from_str(&text)?;
    Ok(state)
}

pub fn save_project_editor_state(
    project_root: &Path,
    state: &ProjectEditorState,
) -> anyhow::Result<()> {
    let path = editor_state_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create editor dir {:?}", parent))?;
    }
    let text = toml::to_string_pretty(state)?;
    fs::write(&path, text).with_context(|| format!("write editor state {:?}", path))?;
    Ok(())
}

pub fn save_project_state_on_change(state: Res<ProjectEditorStateResource>) {
    if !state.is_changed() {
        return;
    }
    let Some(root) = &state.root else {
        return;
    };
    if let Err(err) = save_project_editor_state(root, &state.state) {
        warn!("failed to save editor state: {err}");
    }
}

fn editor_state_path(project_root: &Path) -> PathBuf {
    project_root.join(EDITOR_DIR_NAME).join(EDITOR_STATE_FILE)
}
