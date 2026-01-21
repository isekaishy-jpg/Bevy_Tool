use std::path::{Path, PathBuf};

use world::schema::RegionBounds;

use super::ProjectPanelState;

pub(super) fn sync_from_project(
    state: &mut ProjectPanelState,
    info: &editor_core::project::ProjectInfo,
) {
    let project_key = info.root.to_string_lossy().to_string();
    let world_key = info
        .current_world_id
        .clone()
        .unwrap_or_else(|| "none".to_string());
    if state.project_key.as_deref() == Some(&project_key)
        && state.world_key.as_deref() == Some(&world_key)
    {
        return;
    }

    state.project_key = Some(project_key);
    state.world_key = Some(world_key);
    state.project_name_edit = info.manifest.project_name.clone();

    if let Some(world) = info.current_world() {
        state.world_name_edit = world.manifest.world_name.clone();
        state.tile_size_meters = world.manifest.world_spec.tile_size_meters;
        state.chunks_per_tile = world.manifest.world_spec.chunks_per_tile;
        state.heightfield_samples = world.manifest.world_spec.heightfield_samples;
        state.weightmap_resolution = world.manifest.world_spec.weightmap_resolution;
        state.liquids_resolution = world.manifest.world_spec.liquids_resolution;
    }
}

pub(super) fn world_spec_from_state(state: &ProjectPanelState) -> world::schema::WorldSpec {
    world::schema::WorldSpec {
        tile_size_meters: state.tile_size_meters,
        chunks_per_tile: state.chunks_per_tile,
        heightfield_samples: state.heightfield_samples,
        weightmap_resolution: state.weightmap_resolution,
        liquids_resolution: state.liquids_resolution,
    }
}

pub(super) fn is_world_dirty(
    state: &ProjectPanelState,
    world: &editor_core::project::WorldInfo,
) -> bool {
    state.world_name_edit.trim() != world.manifest.world_name
        || state.tile_size_meters != world.manifest.world_spec.tile_size_meters
        || state.chunks_per_tile != world.manifest.world_spec.chunks_per_tile
        || state.heightfield_samples != world.manifest.world_spec.heightfield_samples
        || state.weightmap_resolution != world.manifest.world_spec.weightmap_resolution
        || state.liquids_resolution != world.manifest.world_spec.liquids_resolution
}

pub(super) fn world_manifest_from_state(
    state: &ProjectPanelState,
    world: &editor_core::project::WorldInfo,
) -> world::schema::WorldManifest {
    let mut updated = world.manifest.clone();
    let world_name = state.world_name_edit.trim();
    updated.world_name = if world_name.is_empty() {
        world.manifest.world_name.clone()
    } else {
        world_name.to_string()
    };
    updated.world_spec = world_spec_from_state(state);
    updated
}

pub(super) fn can_add_region(
    state: &ProjectPanelState,
    world: &editor_core::project::WorldInfo,
) -> bool {
    let region_id = state.new_region_id.trim();
    let region_name = state.new_region_name.trim();
    if region_id.is_empty() || region_name.is_empty() {
        return false;
    }
    let bounds = RegionBounds::new(
        state.region_min_x,
        state.region_min_y,
        state.region_max_x,
        state.region_max_y,
    );
    if !bounds.is_valid() {
        return false;
    }
    !world
        .manifest
        .regions
        .iter()
        .any(|region| region.region_id == region_id)
}

pub(super) fn can_create_project(state: &ProjectPanelState) -> bool {
    if state.new_project_path.trim().is_empty() && state.new_project_name.trim().is_empty() {
        return false;
    }
    let bounds = RegionBounds::new(
        state.region_min_x,
        state.region_min_y,
        state.region_max_x,
        state.region_max_y,
    );
    if !bounds.is_valid() {
        return false;
    }
    !state.new_region_id.trim().is_empty() && !state.new_region_name.trim().is_empty()
}

pub(super) fn resolve_project_root(state: &ProjectPanelState) -> PathBuf {
    let trimmed = state.new_project_path.trim();
    if !trimmed.is_empty() {
        return PathBuf::from(trimmed);
    }

    let base_dir = default_projects_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let folder_name = sanitized_name(state.new_project_name.trim());
    let preferred = base_dir.join(folder_name);
    unique_project_root(preferred)
}

fn default_projects_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        let home = std::env::var("USERPROFILE").ok()?;
        return Some(
            PathBuf::from(home)
                .join("Documents")
                .join("BevyTool")
                .join("Projects"),
        );
    }

    let home = std::env::var("HOME").ok()?;
    if cfg!(target_os = "macos") {
        return Some(
            PathBuf::from(home)
                .join("Documents")
                .join("BevyTool")
                .join("Projects"),
        );
    }

    Some(PathBuf::from(home).join("BevyTool").join("Projects"))
}

fn sanitized_name(name: &str) -> String {
    let trimmed = if name.is_empty() { "NewProject" } else { name };
    let sanitized = trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string();

    if sanitized.is_empty() {
        "NewProject".to_string()
    } else {
        sanitized
    }
}

fn unique_project_root(preferred: PathBuf) -> PathBuf {
    if !preferred.exists() {
        return preferred;
    }
    let base = preferred
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("NewProject")
        .to_string();
    let parent = preferred.parent().map(Path::to_path_buf);
    for index in 1..100 {
        let candidate = format!("{base}-{index}");
        let Some(parent) = &parent else {
            break;
        };
        let path = parent.join(&candidate);
        if !path.exists() {
            return path;
        }
    }
    preferred
}
