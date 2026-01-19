use crate::schema::{ProjectManifest, WorldManifest};
use anyhow::Context;
use std::fs;
use std::path::Path;

pub const PROJECT_MANIFEST_FILE: &str = "project.toml";
pub const WORLD_MANIFEST_FILE: &str = "world.toml";

pub fn write_project_manifest(
    project_root: &Path,
    manifest: &ProjectManifest,
) -> anyhow::Result<()> {
    fs::create_dir_all(project_root)
        .with_context(|| format!("create project root {:?}", project_root))?;

    let path = project_root.join(PROJECT_MANIFEST_FILE);
    let text = toml::to_string_pretty(manifest)?;
    fs::write(&path, text).with_context(|| format!("write manifest {:?}", path))?;
    Ok(())
}

pub fn read_project_manifest(project_root: &Path) -> anyhow::Result<ProjectManifest> {
    let path = project_root.join(PROJECT_MANIFEST_FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("read manifest {:?}", path))?;
    let manifest = toml::from_str(&text)?;
    Ok(manifest)
}

pub fn write_world_manifest(world_root: &Path, manifest: &WorldManifest) -> anyhow::Result<()> {
    fs::create_dir_all(world_root)
        .with_context(|| format!("create world root {:?}", world_root))?;

    let path = world_root.join(WORLD_MANIFEST_FILE);
    let text = toml::to_string_pretty(manifest)?;
    fs::write(&path, text).with_context(|| format!("write world manifest {:?}", path))?;
    Ok(())
}

pub fn read_world_manifest(world_root: &Path) -> anyhow::Result<WorldManifest> {
    let path = world_root.join(WORLD_MANIFEST_FILE);
    let text =
        fs::read_to_string(&path).with_context(|| format!("read world manifest {:?}", path))?;
    let manifest = toml::from_str(&text)?;
    Ok(manifest)
}
