use crate::schema::{ProjectManifest, WORLD_SCHEMA_VERSION};
use anyhow::{anyhow, bail};

pub struct Migration {
    pub from: u32,
    pub to: u32,
    pub apply: fn(&mut ProjectManifest) -> anyhow::Result<()>,
}

const MIGRATIONS: &[Migration] = &[];

pub fn migrate_manifest(manifest: &mut ProjectManifest) -> anyhow::Result<()> {
    if manifest.format_version > WORLD_SCHEMA_VERSION {
        bail!(
            "manifest format version {} is newer than supported {}",
            manifest.format_version,
            WORLD_SCHEMA_VERSION
        );
    }

    while manifest.format_version < WORLD_SCHEMA_VERSION {
        let from = manifest.format_version;
        let migration = MIGRATIONS
            .iter()
            .find(|entry| entry.from == from)
            .ok_or_else(|| anyhow!("no migration registered from {}", from))?;
        (migration.apply)(manifest)?;
        manifest.format_version = migration.to;
    }

    Ok(())
}
