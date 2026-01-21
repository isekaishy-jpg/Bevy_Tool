use crate::migrations::migrate_project_manifest;
use crate::schema::PROJECT_FORMAT_VERSION;
use crate::storage::{project_layout, read_project_manifest, PROJECT_MANIFEST_FILE};
use serde::Serialize;
use std::path::{Path, PathBuf};

mod tile;
mod world;

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
    let mut issues = Vec::new();

    let manifest = match read_project_manifest(project_root) {
        Ok(manifest) => manifest,
        Err(err) => {
            issues.push(
                ValidationIssue::new(format!("manifest read failed: {err}"))
                    .with_path(project_root.join(PROJECT_MANIFEST_FILE)),
            );
            return issues;
        }
    };

    if manifest.format_version > PROJECT_FORMAT_VERSION {
        issues.push(
            ValidationIssue::new(format!(
                "manifest format version {} exceeds supported {}",
                manifest.format_version, PROJECT_FORMAT_VERSION
            ))
            .with_path(project_root.join(PROJECT_MANIFEST_FILE)),
        );
    }

    if let Err(err) = migrate_project_manifest(&mut manifest.clone()) {
        issues.push(ValidationIssue::new(format!(
            "manifest migration check failed: {err}"
        )));
    }

    let layout = project_layout(project_root, &manifest);
    world::scan_worlds(&layout, quarantine, &mut issues);

    issues
}
