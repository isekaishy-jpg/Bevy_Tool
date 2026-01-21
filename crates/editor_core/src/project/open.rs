use std::fs;
use std::path::Path;

use world::storage::{project_layout, read_project_manifest, read_world_manifest, ProjectLayout};

use crate::editor_state::{load_project_editor_state, ProjectEditorStateResource};

use super::helpers::{choose_world_id, world_has_tiles};
use super::{ProjectInfo, WorldInfo};

pub(super) fn open_project(
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
