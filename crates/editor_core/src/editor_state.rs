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
    pub viewport_overlays: ViewportOverlayPrefs,
}

impl Default for ProjectEditorState {
    fn default() -> Self {
        Self {
            dock_layout: None,
            last_world_id: None,
            autosave_enabled: true,
            viewport_overlays: ViewportOverlayPrefs::default(),
        }
    }
}

/// Persisted viewport overlay preferences stored per project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ViewportOverlayPrefs {
    pub show_cursor_readout: bool,
    pub show_fps: bool,
    pub present_mode: u8,
    pub show_tile_grid: bool,
    pub show_chunk_grid: bool,
    pub show_subgrid: bool,
    pub show_region_bounds: bool,
    pub show_hover_highlight: bool,
    pub show_selection_highlight: bool,
    pub show_debug_markers: bool,
    pub show_streaming: bool,
    pub snap_mode: u8,
    pub subgrid_spacing: u16,
}

impl Default for ViewportOverlayPrefs {
    fn default() -> Self {
        Self {
            show_cursor_readout: true,
            show_fps: true,
            present_mode: 0,
            show_tile_grid: true,
            show_chunk_grid: false,
            show_subgrid: false,
            show_region_bounds: true,
            show_hover_highlight: true,
            show_selection_highlight: true,
            show_debug_markers: true,
            show_streaming: false,
            snap_mode: 0,
            subgrid_spacing: 8,
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
