use crate::schema::ProjectManifest;
use anyhow::Context;
use std::fs;
use std::path::Path;

pub const MANIFEST_FILE: &str = "project.toml";

pub fn write_manifest(project_root: &Path, manifest: &ProjectManifest) -> anyhow::Result<()> {
    fs::create_dir_all(project_root)
        .with_context(|| format!("create project root {:?}", project_root))?;

    let path = project_root.join(MANIFEST_FILE);
    let text = toml::to_string_pretty(manifest)?;
    fs::write(&path, text).with_context(|| format!("write manifest {:?}", path))?;
    Ok(())
}

pub fn read_manifest(project_root: &Path) -> anyhow::Result<ProjectManifest> {
    let path = project_root.join(MANIFEST_FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("read manifest {:?}", path))?;
    let manifest = toml::from_str(&text)?;
    Ok(manifest)
}
