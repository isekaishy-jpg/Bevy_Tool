use crate::migrations::migrate_manifest;
use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{default_layout, quarantine_tile_dir, read_manifest, read_tile_meta};
use foundation::ids::{TileCoord, TileId};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub message: String,
    pub path: Option<PathBuf>,
}

impl ValidationIssue {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }
}

pub fn validate_project(project_root: &Path) -> Vec<ValidationIssue> {
    validate_project_impl(project_root, false)
}

pub fn validate_project_and_quarantine(project_root: &Path) -> Vec<ValidationIssue> {
    validate_project_impl(project_root, true)
}

fn validate_project_impl(project_root: &Path, quarantine: bool) -> Vec<ValidationIssue> {
    let layout = default_layout(project_root);
    let mut issues = Vec::new();

    let manifest = match read_manifest(&layout.project_root) {
        Ok(manifest) => manifest,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("manifest read failed: {err}"))
                    .with_path(layout.project_root.join(crate::storage::MANIFEST_FILE)),
            );
            return issues;
        }
    };

    if manifest.format_version > WORLD_SCHEMA_VERSION {
        issues.push(
            ValidationIssue::new(format!(
                "manifest format version {} exceeds supported {}",
                manifest.format_version, WORLD_SCHEMA_VERSION
            ))
            .with_path(layout.project_root.join(crate::storage::MANIFEST_FILE)),
        );
    }

    if let Err(err) = migrate_manifest(&mut manifest.clone()) {
        issues.push(ValidationIssue::new(format!(
            "migration check failed: {err}"
        )));
    }

    scan_tiles(&layout, quarantine, &mut issues);

    issues
}

fn scan_tiles(
    layout: &crate::storage::Layout,
    quarantine: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    if !layout.tiles_dir.exists() {
        return;
    }

    let regions = match std::fs::read_dir(&layout.tiles_dir) {
        Ok(entries) => entries,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("read tiles dir failed: {err}"))
                    .with_path(layout.tiles_dir.clone()),
            );
            return;
        }
    };

    for region_entry in regions.flatten() {
        let region_path = region_entry.path();
        if !region_path.is_dir() {
            continue;
        }

        let region_name = match region_path.file_name().and_then(|name| name.to_str()) {
            Some(name) if name != "_quarantine" => name.to_string(),
            _ => continue,
        };

        let tiles = match std::fs::read_dir(&region_path) {
            Ok(entries) => entries,
            Err(err) => {
                issues.push(
                    ValidationIssue::new(format!("read region failed: {err}"))
                        .with_path(region_path.clone()),
                );
                continue;
            }
        };

        for tile_entry in tiles.flatten() {
            let tile_path = tile_entry.path();
            if !tile_path.is_dir() {
                continue;
            }

            let tile_name = match tile_path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let tile_id = match parse_tile_id(tile_name) {
                Some(tile_id) => tile_id,
                None => {
                    issues.push(
                        ValidationIssue::new(format!("invalid tile dir name: {tile_name}"))
                            .with_path(tile_path.clone()),
                    );
                    continue;
                }
            };

            if let Err(err) = read_tile_meta(layout, &region_name, tile_id) {
                issues.push(
                    ValidationIssue::new(format!("tile meta read failed: {err}"))
                        .with_path(tile_path.clone()),
                );

                if quarantine {
                    let _ =
                        quarantine_tile_dir(layout, &region_name, tile_id, "tile meta read failed");
                }
            }
        }
    }
}

fn parse_tile_id(name: &str) -> Option<TileId> {
    let mut parts = name.split('_');
    let x = parts.next()?.parse::<i32>().ok()?;
    let y = parts.next()?.parse::<i32>().ok()?;
    Some(TileId {
        coord: TileCoord { x, y },
    })
}
