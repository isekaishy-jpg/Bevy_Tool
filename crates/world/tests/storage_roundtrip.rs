use std::fs;
use std::path::Path;

use tempfile::tempdir;
use world::schema::{ProjectManifest, WORLD_SCHEMA_VERSION};
use world::storage::{
    create_project, load_tile_stub, read_manifest, save_tile_stub, tile_container_path, LiquidBody,
    LiquidKind, LiquidsMask, LiquidsMeta, PropInstance, PropsInstances, TerrainHeight, TileMeta,
    TileStub,
};
use world::{AssetId, InstanceId, TileCoord, TileId};

#[test]
fn create_save_reload_roundtrip() {
    let temp = tempdir().expect("tempdir");
    let manifest = ProjectManifest {
        world_name: "TestWorld".to_string(),
        ..Default::default()
    };

    let layout = create_project(temp.path(), &manifest).expect("create project");

    let tile_id = TileId {
        coord: TileCoord { x: 1, y: -2 },
    };
    let meta = TileMeta::new(tile_id);
    let terrain = TerrainHeight::new(3, (0..9).map(|v| v as f32).collect());
    let liquids_mask = LiquidsMask::new(
        4,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    );
    let liquids_meta = LiquidsMeta::new(vec![LiquidBody {
        id: 1,
        height: 2.5,
        kind: LiquidKind::Water,
    }]);
    let props = PropsInstances::new(vec![PropInstance {
        id: InstanceId(42),
        asset: AssetId::new("core", "tree_oak"),
        translation: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }]);

    let stub = TileStub {
        meta: meta.clone(),
        terrain: terrain.clone(),
        liquids_mask: liquids_mask.clone(),
        liquids_meta: liquids_meta.clone(),
        props: props.clone(),
    };

    save_tile_stub(&layout, "region_0", tile_id, &stub).expect("save tile stub");
    let loaded = load_tile_stub(&layout, "region_0", tile_id).expect("load tile stub");

    assert_eq!(loaded.meta, meta);
    assert_eq!(loaded.terrain, terrain);
    assert_eq!(loaded.liquids_mask, liquids_mask);
    assert_eq!(loaded.liquids_meta, liquids_meta);
    assert_eq!(loaded.props, props);

    let loaded_manifest = read_manifest(temp.path()).expect("read manifest");
    assert_eq!(loaded_manifest, manifest);
}

#[test]
fn validator_flags_newer_manifest_version() {
    let temp = tempdir().expect("tempdir");
    let manifest = ProjectManifest {
        format_version: WORLD_SCHEMA_VERSION + 1,
        ..Default::default()
    };

    create_project(temp.path(), &manifest).expect("create project");

    let issues = world::validator::validate_project(temp.path());
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("format version")),
        "expected format version issue"
    );
}

#[test]
fn validator_quarantines_corrupt_tile() {
    let temp = tempdir().expect("tempdir");
    let manifest = ProjectManifest::default();
    let layout = create_project(temp.path(), &manifest).expect("create project");

    let tile_id = TileId {
        coord: TileCoord { x: 0, y: 0 },
    };
    let tile_path = tile_container_path(&layout, "region_0", tile_id);
    if let Some(parent) = tile_path.parent() {
        fs::create_dir_all(parent).expect("create tile dir");
    }
    fs::write(&tile_path, b"not-a-tile").expect("write corrupt tile");

    let issues = world::validator::validate_project_and_quarantine(temp.path());
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("tile header read failed")),
        "expected tile header read failure"
    );

    assert!(
        has_quarantined_tile(&layout.quarantine_dir),
        "expected quarantined tile"
    );
}

fn has_quarantined_tile(root: &Path) -> bool {
    let Ok(timestamps) = fs::read_dir(root) else {
        return false;
    };
    for timestamp in timestamps.flatten() {
        let Ok(regions) = fs::read_dir(timestamp.path()) else {
            continue;
        };
        for region in regions.flatten() {
            let Ok(tiles) = fs::read_dir(region.path()) else {
                continue;
            };
            for tile in tiles.flatten() {
                if tile.path().extension().and_then(|ext| ext.to_str()) == Some("tile") {
                    return true;
                }
            }
        }
    }
    false
}
