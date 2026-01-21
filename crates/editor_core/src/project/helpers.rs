use std::fs;
use std::path::Path;

use bevy::log::warn;
use world::schema::{ProjectManifest, WorldManifest};
use world::storage::{
    create_region, project_layout, region_tiles_dir, world_layout, ProjectLayout,
};

use super::{ProjectInfo, WorldInfo};
use crate::EditorConfig;

pub(super) fn world_has_tiles(layout: &ProjectLayout, manifest: &WorldManifest) -> bool {
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

pub(super) fn ensure_world_regions(root: &Path, project: &ProjectManifest, world: &WorldManifest) {
    let layout = project_layout(root, project);
    let world_layout = world_layout(&layout, &world.world_id);
    for region in &world.regions {
        if let Err(err) = create_region(&world_layout, region) {
            warn!("failed to create region {}: {err}", region.region_id);
        }
    }
}

pub(super) fn choose_world_id(worlds: &[WorldInfo], preferred: Option<&str>) -> Option<String> {
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

pub(super) fn update_config(config: &mut EditorConfig, info: &ProjectInfo) {
    config.project_name = info.manifest.project_name.clone();
    config.world_name = info
        .current_world()
        .map(|world| world.manifest.world_name.clone())
        .unwrap_or_else(|| "Untitled".to_string());
}
