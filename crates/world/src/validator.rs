use crate::migrations::migrate_manifest;
use crate::schema::WORLD_SCHEMA_VERSION;
use crate::storage::{default_layout, quarantine_tile_file, read_manifest};
use crate::tile_container::world_spec_hash::{hash_region, hash_world_spec, DEFAULT_WORLD_SPEC};
use crate::tile_container::{
    decode_hmap, decode_liqd, decode_meta, decode_prop, decode_wmap, TileContainerReader,
    TileSectionTag, CONTAINER_VERSION, DEFAULT_ALIGNMENT, DIR_ENTRY_SIZE, HEADER_SIZE,
    MAX_SECTION_COUNT, MIN_CONTAINER_VERSION,
};
use foundation::ids::{TileCoord, TileId};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
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

pub fn validate_project_json(project_root: &Path) -> anyhow::Result<String> {
    let issues = validate_project(project_root);
    Ok(serde_json::to_string_pretty(&issues)?)
}

pub fn validate_project_and_quarantine_json(project_root: &Path) -> anyhow::Result<String> {
    let issues = validate_project_and_quarantine(project_root);
    Ok(serde_json::to_string_pretty(&issues)?)
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
            if tile_path.extension().and_then(|ext| ext.to_str()) != Some("tile") {
                continue;
            }

            let tile_name = match tile_path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let tile_id = match parse_tile_filename(tile_name) {
                Some(tile_id) => tile_id,
                None => {
                    issues.push(
                        ValidationIssue::new(format!("invalid tile filename: {tile_name}"))
                            .with_path(tile_path.clone()),
                    );
                    continue;
                }
            };

            validate_tile_container(
                layout,
                &region_name,
                tile_id,
                &tile_path,
                quarantine,
                issues,
            );
        }
    }
}

fn parse_tile_filename(name: &str) -> Option<TileId> {
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

fn validate_tile_container(
    layout: &crate::storage::Layout,
    region: &str,
    tile_id: TileId,
    tile_path: &Path,
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

    let expected_spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    if reader.header.world_spec_hash != expected_spec_hash {
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

    validate_directory(&reader, tile_path, issues);
    validate_sections(&reader, tile_path, issues);

    if quarantine
        && issues
            .iter()
            .any(|issue| issue.path.as_deref() == Some(tile_path))
    {
        let _ = quarantine_tile_file(layout, region, tile_id, "tile validation failed");
    }
}

fn validate_directory(
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

fn validate_sections(
    reader: &TileContainerReader,
    tile_path: &Path,
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
                Ok(hmap) => validate_hmap(&hmap, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("HMAP decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            tag if tag == TileSectionTag::WMAP => match decode_wmap(&payload) {
                Ok(wmap) => validate_wmap(&wmap, tile_path, issues),
                Err(err) => issues.push(
                    ValidationIssue::new(format!("WMAP decode failed: {err}"))
                        .with_path(tile_path.to_path_buf()),
                ),
            },
            tag if tag == TileSectionTag::LIQD => match decode_liqd(&payload) {
                Ok(liqd) => validate_liqd(&liqd, tile_path, issues),
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

fn validate_hmap(
    hmap: &crate::tile_container::HmapSection,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if hmap.width != DEFAULT_WORLD_SPEC.heightfield_samples
        || hmap.height != DEFAULT_WORLD_SPEC.heightfield_samples
    {
        issues.push(
            ValidationIssue::new("HMAP dimensions do not match world spec")
                .with_path(tile_path.to_path_buf()),
        );
    }
    for sample in &hmap.samples {
        if !sample.is_finite() || *sample < -500.0 || *sample > 5000.0 {
            issues.push(
                ValidationIssue::new("HMAP sample out of range").with_path(tile_path.to_path_buf()),
            );
            break;
        }
    }
}

fn validate_wmap(
    wmap: &crate::tile_container::WmapSection,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if wmap.width != DEFAULT_WORLD_SPEC.weightmap_resolution
        || wmap.height != DEFAULT_WORLD_SPEC.weightmap_resolution
    {
        issues.push(
            ValidationIssue::new("WMAP dimensions do not match world spec")
                .with_path(tile_path.to_path_buf()),
        );
    }
}

fn validate_liqd(
    liqd: &crate::tile_container::LiqdSection,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if liqd.width != DEFAULT_WORLD_SPEC.liquids_resolution
        || liqd.height != DEFAULT_WORLD_SPEC.liquids_resolution
    {
        issues.push(
            ValidationIssue::new("LIQD dimensions do not match world spec")
                .with_path(tile_path.to_path_buf()),
        );
    }
    let body_count = liqd.bodies.len() as u8;
    if body_count > 0 {
        for value in &liqd.mask {
            if *value >= body_count {
                issues.push(
                    ValidationIssue::new("LIQD mask references unknown body")
                        .with_path(tile_path.to_path_buf()),
                );
                break;
            }
        }
    }
    for body in &liqd.bodies {
        if !body.height.is_finite() || body.height < -500.0 || body.height > 5000.0 {
            issues.push(
                ValidationIssue::new("LIQD body height out of range")
                    .with_path(tile_path.to_path_buf()),
            );
            break;
        }
    }
}

fn validate_prop(
    prop: &crate::tile_container::PropSection,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    for instance in &prop.instances {
        for value in instance
            .translation
            .iter()
            .chain(instance.rotation.iter())
            .chain(instance.scale.iter())
        {
            if !value.is_finite() {
                issues.push(
                    ValidationIssue::new("PROP transform contains NaN/inf")
                        .with_path(tile_path.to_path_buf()),
                );
                return;
            }
        }
    }
}

fn is_ascii_tag(tag: TileSectionTag) -> bool {
    let bytes = tag.as_bytes();
    bytes
        .iter()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}
