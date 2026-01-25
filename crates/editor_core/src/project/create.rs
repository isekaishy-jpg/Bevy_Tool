use std::path::Path;

use world::schema::{ProjectManifest, RegionManifest, WorldManifest};
use world::storage::{create_project, create_world, project_layout};

use crate::editor_state::ProjectEditorStateResource;

use super::helpers::world_has_tiles;
use super::{NewProjectRequest, NewWorldRequest, ProjectInfo, WorldInfo};

pub(super) fn create_new_project(
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

pub(super) fn create_new_world(
    root: &Path,
    project_manifest: &ProjectManifest,
    request: &NewWorldRequest,
) -> anyhow::Result<WorldInfo> {
    if !request.region_bounds.is_valid() {
        return Err(anyhow::anyhow!("region bounds are invalid"));
    }

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

    let layout = project_layout(root, project_manifest);
    let world_layout = create_world(&layout, &world_manifest)?;
    let world_info = WorldInfo {
        root: world_layout.world_root.clone(),
        manifest: world_manifest.clone(),
        has_tiles: world_has_tiles(&layout, &world_manifest),
    };

    Ok(world_info)
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
