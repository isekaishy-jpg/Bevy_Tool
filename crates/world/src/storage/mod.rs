use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use foundation::ids::TileId;

use crate::schema::{ProjectManifest, RegionManifest, WorldManifest};
use crate::tile_container::{
    encode_hmap, encode_liqd, encode_meta, encode_prop, TileContainerHeader, TileContainerWriter,
    TileSectionPayload, TileSectionTag, DEFAULT_ALIGNMENT,
};

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
pub use manifest::{
    read_project_manifest, read_world_manifest, write_project_manifest, write_world_manifest,
    PROJECT_MANIFEST_FILE, WORLD_MANIFEST_FILE,
};
pub use props::{read_props_instances, write_props_instances, PropInstance, PropsInstances};
pub use quarantine::{quarantine_tile_dir, quarantine_tile_file};
pub use terrain::{read_terrain_height, write_terrain_height, TerrainHeight};
pub use tile_meta::{read_tile_meta, write_tile_meta, TileMeta};

const EDITOR_DIR_NAME: &str = ".editor";

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub project_root: PathBuf,
    pub editor_dir: PathBuf,
    pub worlds_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct WorldLayout {
    pub project_root: PathBuf,
    pub world_id: String,
    pub world_root: PathBuf,
    pub regions_dir: PathBuf,
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

pub fn project_layout(project_root: &Path, manifest: &ProjectManifest) -> ProjectLayout {
    ProjectLayout {
        project_root: project_root.to_path_buf(),
        editor_dir: project_root.join(EDITOR_DIR_NAME),
        worlds_dir: project_root.join(&manifest.worlds_dir),
        assets_dir: project_root.join(&manifest.assets_dir),
        exports_dir: project_root.join(&manifest.exports_dir),
        cache_dir: project_root.join(&manifest.cache_dir),
    }
}

pub fn world_layout(project_layout: &ProjectLayout, world_id: &str) -> WorldLayout {
    let world_root = project_layout.worlds_dir.join(world_id);
    let regions_dir = world_root.join("regions");
    WorldLayout {
        project_root: project_layout.project_root.clone(),
        world_id: world_id.to_string(),
        world_root: world_root.clone(),
        regions_dir: regions_dir.clone(),
        quarantine_dir: regions_dir.join("_quarantine"),
    }
}

pub fn create_project(
    project_root: &Path,
    manifest: &ProjectManifest,
) -> anyhow::Result<ProjectLayout> {
    let layout = project_layout(project_root, manifest);
    write_project_manifest(&layout.project_root, manifest)?;
    fs::create_dir_all(&layout.editor_dir)
        .with_context(|| format!("create editor dir {:?}", layout.editor_dir))?;
    fs::create_dir_all(&layout.worlds_dir)
        .with_context(|| format!("create worlds dir {:?}", layout.worlds_dir))?;
    fs::create_dir_all(&layout.assets_dir)
        .with_context(|| format!("create assets dir {:?}", layout.assets_dir))?;
    fs::create_dir_all(&layout.exports_dir)
        .with_context(|| format!("create exports dir {:?}", layout.exports_dir))?;
    fs::create_dir_all(&layout.cache_dir)
        .with_context(|| format!("create cache dir {:?}", layout.cache_dir))?;
    Ok(layout)
}

pub fn create_world(
    project_layout: &ProjectLayout,
    manifest: &WorldManifest,
) -> anyhow::Result<WorldLayout> {
    let layout = world_layout(project_layout, &manifest.world_id);
    write_world_manifest(&layout.world_root, manifest)?;
    fs::create_dir_all(&layout.regions_dir)
        .with_context(|| format!("create regions dir {:?}", layout.regions_dir))?;
    fs::create_dir_all(&layout.quarantine_dir)
        .with_context(|| format!("create quarantine dir {:?}", layout.quarantine_dir))?;
    for region in &manifest.regions {
        ensure_region_tiles_dir(&layout, &region.region_id)?;
    }
    Ok(layout)
}

pub fn ensure_region_tiles_dir(layout: &WorldLayout, region_id: &str) -> anyhow::Result<PathBuf> {
    let dir = region_tiles_dir(layout, region_id);
    fs::create_dir_all(&dir).with_context(|| format!("create region tiles dir {:?}", dir))?;
    Ok(dir)
}

pub fn create_region(layout: &WorldLayout, region: &RegionManifest) -> anyhow::Result<PathBuf> {
    ensure_region_tiles_dir(layout, &region.region_id)
}

pub fn region_root(layout: &WorldLayout, region_id: &str) -> PathBuf {
    layout.regions_dir.join(region_id)
}

pub fn region_tiles_dir(layout: &WorldLayout, region_id: &str) -> PathBuf {
    region_root(layout, region_id).join("tiles")
}

pub fn tile_container_path(layout: &WorldLayout, region: &str, tile_id: TileId) -> PathBuf {
    region_tiles_dir(layout, region).join(format!("x{}_y{}.tile", tile_id.coord.x, tile_id.coord.y))
}

pub(crate) fn tile_dir(layout: &WorldLayout, region: &str, tile_id: TileId) -> PathBuf {
    region_tiles_dir(layout, region).join(format!("{}_{}", tile_id.coord.x, tile_id.coord.y))
}

pub(crate) fn ensure_tile_dir(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<PathBuf> {
    let dir = tile_dir(layout, region, tile_id);
    fs::create_dir_all(&dir).with_context(|| format!("create tile dir {:?}", dir))?;
    Ok(dir)
}

pub fn save_tile_stub(
    layout: &WorldLayout,
    manifest: &WorldManifest,
    region: &str,
    tile_id: TileId,
    stub: &TileStub,
) -> anyhow::Result<()> {
    let region_hash = crate::tile_container::world_spec_hash::hash_region(region);
    let meta = crate::tile_container::MetaSection {
        format_version: stub.meta.format_version,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let hmap = crate::tile_container::HmapSection {
        width: stub.terrain.size,
        height: stub.terrain.size,
        samples: stub.terrain.samples.clone(),
    };
    let liqd = crate::tile_container::LiqdSection {
        width: stub.liquids_mask.size,
        height: stub.liquids_mask.size,
        mask: stub.liquids_mask.mask.clone(),
        bodies: stub
            .liquids_meta
            .bodies
            .iter()
            .map(|body| crate::tile_container::LiqdBody {
                id: body.id,
                height: body.height,
                kind: match &body.kind {
                    LiquidKind::Water => crate::tile_container::LiqdKind::Water,
                    LiquidKind::Lava => crate::tile_container::LiqdKind::Lava,
                    LiquidKind::Slime => crate::tile_container::LiqdKind::Slime,
                    LiquidKind::Custom(name) => {
                        crate::tile_container::LiqdKind::Custom(name.clone())
                    }
                },
            })
            .collect(),
    };
    let prop = crate::tile_container::PropSection {
        instances: stub
            .props
            .instances
            .iter()
            .map(|instance| crate::tile_container::PropRecord {
                id: instance.id,
                asset: instance.asset.clone(),
                translation: instance.translation,
                rotation: instance.rotation,
                scale: instance.scale,
            })
            .collect(),
    };

    let mut writer = TileContainerWriter::new().alignment(DEFAULT_ALIGNMENT);
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::META,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_meta(&meta),
    });
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::HMAP,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_hmap(&hmap),
    });
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::LIQD,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_liqd(&liqd),
    });
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::PROP,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_prop(&prop)?,
    });

    let spec_hash = crate::tile_container::world_spec_hash::hash_world_spec_from_manifest(manifest);
    let mut header =
        TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);
    header.created_timestamp = meta.created_timestamp;
    let path = tile_container_path(layout, region, tile_id);
    writer.write(path, header)?;
    Ok(())
}

pub fn load_tile_stub(
    layout: &WorldLayout,
    region: &str,
    tile_id: TileId,
) -> anyhow::Result<TileStub> {
    let path = tile_container_path(layout, region, tile_id);
    let reader = crate::tile_container::TileContainerReader::open(&path)?;
    let meta = crate::tile_container::decode_meta(&reader.decode_section(TileSectionTag::META)?)?;
    let hmap = crate::tile_container::decode_hmap(&reader.decode_section(TileSectionTag::HMAP)?)?;
    let liqd = crate::tile_container::decode_liqd(&reader.decode_section(TileSectionTag::LIQD)?)?;
    let prop = crate::tile_container::decode_prop(&reader.decode_section(TileSectionTag::PROP)?)?;

    Ok(TileStub {
        meta: TileMeta {
            format_version: meta.format_version,
            tile_id,
        },
        terrain: TerrainHeight::new(hmap.width, hmap.samples),
        liquids_mask: LiquidsMask::new(liqd.width, liqd.mask),
        liquids_meta: LiquidsMeta::new(
            liqd.bodies
                .into_iter()
                .map(|body| LiquidBody {
                    id: body.id,
                    height: body.height,
                    kind: match body.kind {
                        crate::tile_container::LiqdKind::Water => LiquidKind::Water,
                        crate::tile_container::LiqdKind::Lava => LiquidKind::Lava,
                        crate::tile_container::LiqdKind::Slime => LiquidKind::Slime,
                        crate::tile_container::LiqdKind::Custom(name) => LiquidKind::Custom(name),
                    },
                })
                .collect(),
        ),
        props: PropsInstances::new(
            prop.instances
                .into_iter()
                .map(|instance| PropInstance {
                    id: instance.id,
                    asset: instance.asset,
                    translation: instance.translation,
                    rotation: instance.rotation,
                    scale: instance.scale,
                })
                .collect(),
        ),
    })
}
