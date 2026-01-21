use crate::schema::WorldSpec;
use crate::tile_container::{
    decode_hmap, decode_liqd, decode_meta, decode_prop, decode_wmap, TileContainerReader,
    TileSectionTag,
};
use std::path::Path;

use super::checks::{validate_hmap, validate_liqd, validate_prop, validate_wmap};
use super::ValidationIssue;

pub(super) fn validate_sections(
    reader: &TileContainerReader,
    tile_path: &Path,
    expected_spec: WorldSpec,
    issues: &mut Vec<ValidationIssue>,
) {
    if reader.section(TileSectionTag::META).is_none() {
        issues
            .push(ValidationIssue::new("missing META section").with_path(tile_path.to_path_buf()));
    }

    for entry in &reader.directory {
        let payload = match reader.decode_section(entry.tag) {
            Ok(payload) => payload,
            Err(err) => {
                issues.push(
                    ValidationIssue::new(format!("section {} read failed: {err}", entry.tag))
                        .with_path(tile_path.to_path_buf()),
                );
                continue;
            }
        };

        match entry.tag {
            tag if tag == TileSectionTag::META => {
                if let Err(err) = decode_meta(&payload) {
                    issues.push(
                        ValidationIssue::new(format!("META decode failed: {err}"))
                            .with_path(tile_path.to_path_buf()),
                    );
                }
            }
            tag if tag == TileSectionTag::HMAP => match decode_hmap(&payload) {
                Ok(hmap) => validate_hmap(&hmap, expected_spec, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("HMAP decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            tag if tag == TileSectionTag::WMAP => match decode_wmap(&payload) {
                Ok(wmap) => validate_wmap(&wmap, expected_spec, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("WMAP decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            tag if tag == TileSectionTag::LIQD => match decode_liqd(&payload) {
                Ok(liqd) => validate_liqd(&liqd, expected_spec, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("LIQD decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            tag if tag == TileSectionTag::PROP => match decode_prop(&payload) {
                Ok(prop) => validate_prop(&prop, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("PROP decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            _ => {}
        }
    }
}
