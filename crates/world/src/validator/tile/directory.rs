use crate::tile_container::{
    TileContainerReader, TileSectionTag, DEFAULT_ALIGNMENT, DIR_ENTRY_SIZE,
};
use std::path::Path;

use super::ValidationIssue;

pub(super) fn validate_directory(
    reader: &TileContainerReader,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    let dir_end = reader.header.section_dir_offset
        + reader.header.section_count as u64 * DIR_ENTRY_SIZE as u64;
    let mut ranges = Vec::new();
    for entry in &reader.directory {
        if !is_ascii_tag(entry.tag) {
            issues.push(
                ValidationIssue::new(format!("section tag {} is not ASCII FourCC", entry.tag))
                    .with_path(tile_path.to_path_buf()),
            );
        }
        if entry.offset < dir_end {
            issues.push(
                ValidationIssue::new(format!("section {} overlaps directory region", entry.tag))
                    .with_path(tile_path.to_path_buf()),
            );
        }
        if entry.stored_len == 0 {
            issues.push(
                ValidationIssue::new(format!("section {} has zero length", entry.tag))
                    .with_path(tile_path.to_path_buf()),
            );
        }
        let end = entry.offset.saturating_add(entry.stored_len);
        if end > reader.file_len {
            issues.push(
                ValidationIssue::new(format!("section {} out of bounds", entry.tag))
                    .with_path(tile_path.to_path_buf()),
            );
        }
        if entry.offset % DEFAULT_ALIGNMENT != 0 {
            issues.push(
                ValidationIssue::new(format!("section {} not aligned", entry.tag))
                    .with_path(tile_path.to_path_buf()),
            );
        }
        ranges.push((entry.offset, end, entry.tag));
    }

    ranges.sort_by_key(|range| range.0);
    for window in ranges.windows(2) {
        let a = &window[0];
        let b = &window[1];
        if b.0 < a.1 {
            issues.push(
                ValidationIssue::new(format!("section overlap: {} overlaps {}", a.2, b.2))
                    .with_path(tile_path.to_path_buf()),
            );
        }
    }
}

fn is_ascii_tag(tag: TileSectionTag) -> bool {
    let bytes = tag.as_bytes();
    bytes
        .iter()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}
