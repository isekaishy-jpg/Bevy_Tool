use crate::migrations::migrate_world_manifest;
use crate::schema::{RegionManifest, WorldManifest, WorldSpec};
use crate::storage::{
    read_world_manifest, region_tiles_dir, world_layout, ProjectLayout, WorldLayout,
    WORLD_MANIFEST_FILE,
};
use crate::tile_container::world_spec_hash::{
    hash_world_spec_from_manifest, hash_world_spec_legacy,
};
use std::collections::HashSet;

use super::tile;
use super::ValidationIssue;

pub(super) fn scan_worlds(
    layout: &ProjectLayout,
    quarantine: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    if !layout.worlds_dir.exists() {
        issues.push(
            ValidationIssue::new("worlds directory missing").with_path(layout.worlds_dir.clone()),
        );
        return;
    }

    let worlds = match std::fs::read_dir(&layout.worlds_dir) {
        Ok(entries) => entries,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("read worlds dir failed: {err}"))
                    .with_path(layout.worlds_dir.clone()),
            );
            return;
        }
    };

    for entry in worlds.flatten() {
        let world_root = entry.path();
        if !world_root.is_dir() {
            continue;
        }

        let dir_name = match world_root.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let world_manifest = match read_world_manifest(&world_root) {
            Ok(manifest) => manifest,
            Err(err) => {
                issues.push(
                    ValidationIssue::new(format!("world manifest read failed: {err}"))
                        .with_path(world_root.join(WORLD_MANIFEST_FILE)),
                );
                continue;
            }
        };

        if world_manifest.world_id != dir_name {
            issues.push(
                ValidationIssue::new("world_id does not match directory name")
                    .with_path(world_root.join(WORLD_MANIFEST_FILE)),
            );
        }

        if let Err(err) = migrate_world_manifest(&mut world_manifest.clone()) {
            issues.push(ValidationIssue::new(format!(
                "world migration check failed: {err}"
            )));
        }

        let world_layout = world_layout(layout, &dir_name);
        scan_world_tiles(&world_layout, &world_manifest, quarantine, issues);
    }
}

pub(super) fn scan_world_tiles(
    layout: &WorldLayout,
    manifest: &WorldManifest,
    quarantine: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    if !layout.regions_dir.exists() {
        issues.push(
            ValidationIssue::new("regions directory missing").with_path(layout.regions_dir.clone()),
        );
        return;
    }

    let mut manifest_regions = HashSet::new();
    for region in &manifest.regions {
        manifest_regions.insert(region.region_id.clone());
        validate_region_entry(layout, region, issues);
    }

    let region_dirs = match std::fs::read_dir(&layout.regions_dir) {
        Ok(entries) => entries,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("read regions dir failed: {err}"))
                    .with_path(layout.regions_dir.clone()),
            );
            return;
        }
    };

    for entry in region_dirs.flatten() {
        let region_path = entry.path();
        if !region_path.is_dir() {
            continue;
        }

        let region_name = match region_path.file_name().and_then(|name| name.to_str()) {
            Some(name) if name != "_quarantine" => name.to_string(),
            _ => continue,
        };

        if !manifest_regions.contains(&region_name) {
            issues.push(
                ValidationIssue::new("region directory not listed in world manifest")
                    .with_path(region_path.clone()),
            );
        }
    }

    let expected_spec_hash = hash_world_spec_from_manifest(manifest);
    let legacy_spec_hash = hash_world_spec_legacy(manifest.world_spec);
    let expected_spec = manifest.world_spec;
    for region in &manifest.regions {
        scan_region_tiles(
            layout,
            region,
            expected_spec_hash,
            legacy_spec_hash,
            expected_spec,
            quarantine,
            issues,
        );
    }
}

fn validate_region_entry(
    layout: &WorldLayout,
    region: &RegionManifest,
    issues: &mut Vec<ValidationIssue>,
) {
    if region.region_id.trim().is_empty() {
        issues.push(ValidationIssue::new("region_id is empty"));
    }
    if !region.bounds.is_valid() {
        issues.push(
            ValidationIssue::new("region bounds are invalid")
                .with_path(layout.world_root.join(WORLD_MANIFEST_FILE)),
        );
    }
}

fn scan_region_tiles(
    layout: &WorldLayout,
    region: &RegionManifest,
    expected_spec_hash: u64,
    legacy_spec_hash: u64,
    expected_spec: WorldSpec,
    quarantine: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    let tiles_dir = region_tiles_dir(layout, &region.region_id);
    if !tiles_dir.exists() {
        issues.push(
            ValidationIssue::new("region tiles directory missing").with_path(tiles_dir.clone()),
        );
        return;
    }

    let tiles = match std::fs::read_dir(&tiles_dir) {
        Ok(entries) => entries,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("read region tiles failed: {err}"))
                    .with_path(tiles_dir.clone()),
            );
            return;
        }
    };

    for tile_entry in tiles.flatten() {
        let tile_path = tile_entry.path();
        if tile_path.extension().and_then(|ext| ext.to_str()) != Some("tile") {
            continue;
        }

        let tile_name = match tile_path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let tile_id = match tile::parse_tile_filename(tile_name) {
            Some(tile_id) => tile_id,
            None => {
                issues.push(
                    ValidationIssue::new(format!("invalid tile filename: {tile_name}"))
                        .with_path(tile_path.clone()),
                );
                continue;
            }
        };

        tile::validate_tile_container(
            layout,
            &region.region_id,
            tile_id,
            &tile_path,
            expected_spec_hash,
            legacy_spec_hash,
            expected_spec,
            quarantine,
            issues,
        );
    }
}
