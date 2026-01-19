use crate::schema::ProjectManifest;
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE: &str = "project.json";

pub fn write_manifest(project_root: &Path, manifest: &ProjectManifest) -> anyhow::Result<()> {
    fs::create_dir_all(project_root)
        .with_context(|| format!("create project root {:?}", project_root))?;

    let path = project_root.join(MANIFEST_FILE);
    let bytes = serde_json::to_vec_pretty(manifest)?;
    fs::write(&path, bytes).with_context(|| format!("write manifest {:?}", path))?;
    Ok(())
}

pub fn read_manifest(project_root: &Path) -> anyhow::Result<ProjectManifest> {
    let path = project_root.join(MANIFEST_FILE);
    let bytes = fs::read(&path).with_context(|| format!("read manifest {:?}", path))?;
    let manifest = serde_json::from_slice(&bytes)?;
    Ok(manifest)
}

pub fn default_layout(project_root: &Path) -> Layout {
    Layout {
        project_root: project_root.to_path_buf(),
        tiles_dir: project_root.join("tiles"),
        cache_dir: project_root.join(".cache"),
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub project_root: PathBuf,
    pub tiles_dir: PathBuf,
    pub cache_dir: PathBuf,
}
