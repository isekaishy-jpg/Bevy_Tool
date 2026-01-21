use crate::schema::WorldSpec;
use crate::storage::{quarantine_tile_file, WorldLayout};
use crate::tile_container::world_spec_hash::hash_region;
use crate::tile_container::{
    TileContainerReader, CONTAINER_VERSION, HEADER_SIZE, MAX_SECTION_COUNT, MIN_CONTAINER_VERSION,
};
use foundation::ids::{TileCoord, TileId};
use std::path::Path;

use super::ValidationIssue;

mod checks;
mod directory;
mod sections;

pub(super) fn parse_tile_filename(name: &str) -> Option<TileId> {
    let stem = name.strip_suffix(".tile")?;
    let mut parts = stem.split('_');
    let x_part = parts.next()?;
    let y_part = parts.next()?;
    if !x_part.starts_with('x') || !y_part.starts_with('y') {
        return None;
    }
    let x = x_part[1..].parse::<i32>().ok()?;
    let y = y_part[1..].parse::<i32>().ok()?;
    Some(TileId {
        coord: TileCoord { x, y },
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_tile_container(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
    tile_path: &Path,
    expected_spec_hash: u64,
    legacy_spec_hash: u64,
    expected_spec: WorldSpec,
    quarantine: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    let reader = match TileContainerReader::open(tile_path) {
        Ok(reader) => reader,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("tile header read failed: {err}"))
                    .with_path(tile_path.to_path_buf()),
            );
            if quarantine {
                let _ = quarantine_tile_file(layout, region, tile_id, "tile header read failed");
            }
            return;
        }
    };

    if reader.header.section_count > MAX_SECTION_COUNT {
        issues.push(
            ValidationIssue::new(format!(
                "section_count {} exceeds cap",
                reader.header.section_count
            ))
            .with_path(tile_path.to_path_buf()),
        );
    }

    if reader.header.container_version < MIN_CONTAINER_VERSION {
        issues.push(
            ValidationIssue::new(format!(
                "container version {} below minimum {}",
                reader.header.container_version, MIN_CONTAINER_VERSION
            ))
            .with_path(tile_path.to_path_buf()),
        );
    }

    if reader.header.container_version > CONTAINER_VERSION {
        issues.push(
            ValidationIssue::new(format!(
                "container version {} exceeds supported {}",
                reader.header.container_version, CONTAINER_VERSION
            ))
            .with_path(tile_path.to_path_buf()),
        );
    }

    if reader.header.section_dir_offset < HEADER_SIZE as u64 {
        issues.push(
            ValidationIssue::new("section directory overlaps header")
                .with_path(tile_path.to_path_buf()),
        );
    }

    let expected_hash = hash_region(region);
    if reader.header.region_hash != expected_hash {
        issues
            .push(ValidationIssue::new("region hash mismatch").with_path(tile_path.to_path_buf()));
    }

    if reader.header.world_spec_hash != expected_spec_hash
        && reader.header.world_spec_hash != legacy_spec_hash
    {
        issues.push(
            ValidationIssue::new("world spec hash mismatch").with_path(tile_path.to_path_buf()),
        );
    }

    if reader.header.tile_x != tile_id.coord.x || reader.header.tile_y != tile_id.coord.y {
        issues.push(
            ValidationIssue::new("tile_id does not match filename")
                .with_path(tile_path.to_path_buf()),
        );
    }

    directory::validate_directory(&reader, tile_path, issues);
    sections::validate_sections(&reader, tile_path, expected_spec, issues);

    if quarantine
        && issues
            .iter()
            .any(|issue| issue.path.as_deref() == Some(tile_path))
    {
        let _ = quarantine_tile_file(layout, region, tile_id, "tile validation failed");
    }
}
