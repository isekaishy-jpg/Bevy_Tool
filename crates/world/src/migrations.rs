use crate::schema::{ProjectManifest, WorldManifest, PROJECT_FORMAT_VERSION, WORLD_FORMAT_VERSION};
use anyhow::{anyhow, bail};

pub struct ProjectMigration {
    pub from: u32,
    pub to: u32,
    pub apply: fn(&mut ProjectManifest) -> anyhow::Result<()>,
}

pub struct WorldMigration {
    pub from: u32,
    pub to: u32,
    pub apply: fn(&mut WorldManifest) -> anyhow::Result<()>,
}

pub const MIN_PROJECT_FORMAT_VERSION: u32 = 1;
pub const MIN_WORLD_FORMAT_VERSION: u32 = 1;

const PROJECT_MIGRATIONS: &[ProjectMigration] = &[];
const WORLD_MIGRATIONS: &[WorldMigration] = &[];

pub fn migrate_project_manifest(manifest: &mut ProjectManifest) -> anyhow::Result<()> {
    if manifest.format_version < MIN_PROJECT_FORMAT_VERSION {
        bail!(
            "manifest format version {} is below minimum {}",
            manifest.format_version,
            MIN_PROJECT_FORMAT_VERSION
        );
    }

    if manifest.format_version > PROJECT_FORMAT_VERSION {
        bail!(
            "manifest format version {} is newer than supported {}",
            manifest.format_version,
            PROJECT_FORMAT_VERSION
        );
    }

    while manifest.format_version < PROJECT_FORMAT_VERSION {
        let from = manifest.format_version;
        let migration = PROJECT_MIGRATIONS
            .iter()
            .find(|entry| entry.from == from)
            .ok_or_else(|| anyhow!("no migration registered from {}", from))?;
        (migration.apply)(manifest)?;
        manifest.format_version = migration.to;
    }

    Ok(())
}

pub fn migrate_world_manifest(manifest: &mut WorldManifest) -> anyhow::Result<()> {
    if manifest.format_version < MIN_WORLD_FORMAT_VERSION {
        bail!(
            "world format version {} is below minimum {}",
            manifest.format_version,
            MIN_WORLD_FORMAT_VERSION
        );
    }

    if manifest.format_version > WORLD_FORMAT_VERSION {
        bail!(
            "world format version {} is newer than supported {}",
            manifest.format_version,
            WORLD_FORMAT_VERSION
        );
    }

    while manifest.format_version < WORLD_FORMAT_VERSION {
        let from = manifest.format_version;
        let migration = WORLD_MIGRATIONS
            .iter()
            .find(|entry| entry.from == from)
            .ok_or_else(|| anyhow!("no migration registered from {}", from))?;
        (migration.apply)(manifest)?;
        manifest.format_version = migration.to;
    }

    Ok(())
}
