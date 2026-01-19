use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use foundation::ids::TileId;

// NOTE: Stub formats currently serialize JSON payloads even for .bin files.

pub mod liquids;
pub mod manifest;
pub mod props;
pub mod quarantine;
pub mod terrain;
pub mod tile_meta;

pub use liquids::{
    read_liquids_mask, read_liquids_meta, write_liquids_mask, write_liquids_meta, LiquidBody,
    LiquidKind, LiquidsMask, LiquidsMeta,
};
pub use manifest::{read_manifest, write_manifest, MANIFEST_FILE};
pub use props::{read_props_instances, write_props_instances, PropInstance, PropsInstances};
pub use quarantine::quarantine_tile_dir;
pub use terrain::{read_terrain_height, write_terrain_height, TerrainHeight};
pub use tile_meta::{read_tile_meta, write_tile_meta, TileMeta};

#[derive(Debug, Clone)]
pub struct Layout {
    pub project_root: PathBuf,
    pub tiles_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub quarantine_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileStub {
    pub meta: TileMeta,
    pub terrain: TerrainHeight,
    pub liquids_mask: LiquidsMask,
    pub liquids_meta: LiquidsMeta,
    pub props: PropsInstances,
}

pub fn default_layout(project_root: &Path) -> Layout {
    let tiles_dir = project_root.join("tiles");
    Layout {
        project_root: project_root.to_path_buf(),
        tiles_dir: tiles_dir.clone(),
        cache_dir: project_root.join(".cache"),
        quarantine_dir: tiles_dir.join("_quarantine"),
    }
}

pub fn create_project(
    project_root: &Path,
    manifest: &crate::schema::ProjectManifest,
) -> anyhow::Result<Layout> {
    let layout = default_layout(project_root);
    write_manifest(&layout.project_root, manifest)?;
    fs::create_dir_all(&layout.tiles_dir)
        .with_context(|| format!("create tiles dir {:?}", layout.tiles_dir))?;
    fs::create_dir_all(&layout.cache_dir)
        .with_context(|| format!("create cache dir {:?}", layout.cache_dir))?;
    fs::create_dir_all(&layout.quarantine_dir)
        .with_context(|| format!("create quarantine dir {:?}", layout.quarantine_dir))?;
    Ok(layout)
}

pub(crate) fn tile_dir(layout: &Layout, region: &str, tile_id: TileId) -> PathBuf {
    layout
        .tiles_dir
        .join(region)
        .join(format!("{}_{}", tile_id.coord.x, tile_id.coord.y))
}

pub(crate) fn ensure_tile_dir(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<PathBuf> {
    let dir = tile_dir(layout, region, tile_id);
    fs::create_dir_all(&dir).with_context(|| format!("create tile dir {:?}", dir))?;
    Ok(dir)
}

pub fn save_tile_stub(
    layout: &Layout,
    region: &str,
    tile_id: TileId,
    stub: &TileStub,
) -> anyhow::Result<()> {
    write_tile_meta(layout, region, tile_id, &stub.meta)?;
    write_terrain_height(layout, region, tile_id, &stub.terrain)?;
    write_liquids_mask(layout, region, tile_id, &stub.liquids_mask)?;
    write_liquids_meta(layout, region, tile_id, &stub.liquids_meta)?;
    write_props_instances(layout, region, tile_id, &stub.props)?;
    Ok(())
}

pub fn load_tile_stub(layout: &Layout, region: &str, tile_id: TileId) -> anyhow::Result<TileStub> {
    Ok(TileStub {
        meta: read_tile_meta(layout, region, tile_id)?,
        terrain: read_terrain_height(layout, region, tile_id)?,
        liquids_mask: read_liquids_mask(layout, region, tile_id)?,
        liquids_meta: read_liquids_meta(layout, region, tile_id)?,
        props: read_props_instances(layout, region, tile_id)?,
    })
}
