use crate::schema::WorldSpec;
use std::path::Path;

use super::ValidationIssue;

pub(super) fn validate_hmap(
    hmap: &crate::tile_container::HmapSection,
    expected_spec: WorldSpec,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if hmap.width != expected_spec.heightfield_samples
        || hmap.height != expected_spec.heightfield_samples
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

pub(super) fn validate_wmap(
    wmap: &crate::tile_container::WmapSection,
    expected_spec: WorldSpec,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if wmap.width != expected_spec.weightmap_resolution
        || wmap.height != expected_spec.weightmap_resolution
    {
        issues.push(
            ValidationIssue::new("WMAP dimensions do not match world spec")
                .with_path(tile_path.to_path_buf()),
        );
    }
}

pub(super) fn validate_liqd(
    liqd: &crate::tile_container::LiqdSection,
    expected_spec: WorldSpec,
    tile_path: &Path,
    issues: &mut Vec<ValidationIssue>,
) {
    if liqd.width != expected_spec.liquids_resolution
        || liqd.height != expected_spec.liquids_resolution
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

pub(super) fn validate_prop(
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
