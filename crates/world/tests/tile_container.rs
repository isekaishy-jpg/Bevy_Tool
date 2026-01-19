use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

use tempfile::tempdir;
use world::schema::{RegionBounds, RegionManifest, WorldManifest, WORLD_FORMAT_VERSION};
use world::storage::{create_project, create_world, tile_container_path};
use world::tile_container::world_spec_hash::{hash_region, hash_world_spec, DEFAULT_WORLD_SPEC};
use world::tile_container::{
    decode_hmap, decode_meta, encode_hmap, encode_meta, encode_prop, HmapSection, MetaSection,
    PropSection, TileContainerHeader, TileContainerReader, TileContainerWriter, TileSectionPayload,
    TileSectionTag, DEFAULT_ALIGNMENT, DIR_ENTRY_SIZE,
};
use world::{TileCoord, TileId};

#[test]
fn tile_container_roundtrip() {
    let temp = tempdir().expect("tempdir");
    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 0, y: 0 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let header = TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let hmap = HmapSection {
        width: 2,
        height: 2,
        samples: vec![0.0, 1.0, 2.0, 3.0],
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

    let path = temp.path().join("x0_y0.tile");
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let read_meta = decode_meta(&reader.decode_section(TileSectionTag::META).unwrap()).unwrap();
    let read_hmap = decode_hmap(&reader.decode_section(TileSectionTag::HMAP).unwrap()).unwrap();

    assert_eq!(read_meta, meta);
    assert_eq!(read_hmap, hmap);
}

#[test]
fn tile_container_deterministic_output() {
    let temp = tempdir().expect("tempdir");
    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 2, y: -4 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let mut header =
        TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);
    header.created_timestamp = 0;

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let hmap = HmapSection {
        width: 2,
        height: 2,
        samples: vec![1.0, 2.0, 3.0, 4.0],
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

    let path_a = temp.path().join("x2_y-4_a.tile");
    let path_b = temp.path().join("x2_y-4_b.tile");
    writer.write(&path_a, header).expect("write tile a");
    writer.write(&path_b, header).expect("write tile b");

    let bytes_a = std::fs::read(&path_a).expect("read tile a");
    let bytes_b = std::fs::read(&path_b).expect("read tile b");
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn tile_container_unknown_section_is_ignored() {
    let temp = tempdir().expect("tempdir");
    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 1, y: 2 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let header = TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
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
        tag: TileSectionTag::from_bytes(*b"JUNK"),
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: b"payload".to_vec(),
    });

    let path = temp.path().join("x1_y2.tile");
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let read_meta = decode_meta(&reader.decode_section(TileSectionTag::META).unwrap()).unwrap();
    assert_eq!(read_meta, meta);
    assert!(reader
        .section(TileSectionTag::from_bytes(*b"JUNK"))
        .is_some());
}

#[test]
fn tile_container_crc_failure_detected() {
    let temp = tempdir().expect("tempdir");
    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: -1, y: 3 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let header = TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let hmap = HmapSection {
        width: 2,
        height: 2,
        samples: vec![0.0, 1.0, 2.0, 3.0],
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

    let path = temp.path().join("x-1_y3.tile");
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let entry = reader.section(TileSectionTag::HMAP).expect("HMAP entry");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .expect("open tile");
    file.seek(SeekFrom::Start(entry.offset))
        .expect("seek to payload");
    file.write_all(&[0xFF]).expect("corrupt payload");
    file.sync_all().expect("sync payload");

    assert!(
        reader.decode_section(TileSectionTag::HMAP).is_err(),
        "expected crc failure"
    );
}

#[test]
fn tile_container_overlap_detected_by_validator() {
    let temp = tempdir().expect("tempdir");
    let project_manifest = world::schema::ProjectManifest::default();
    let project_layout = create_project(temp.path(), &project_manifest).expect("create project");
    let world_manifest = WorldManifest {
        world_id: "world_0".to_string(),
        regions: vec![RegionManifest {
            region_id: "region_0".to_string(),
            name: "Region 0".to_string(),
            bounds: RegionBounds::new(0, 0, 1, 1),
        }],
        ..WorldManifest::default()
    };
    let world_layout = create_world(&project_layout, &world_manifest).expect("create world");

    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 4, y: 5 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let header = TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let prop = PropSection { instances: vec![] };

    let mut writer = TileContainerWriter::new().alignment(DEFAULT_ALIGNMENT);
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::META,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_meta(&meta),
    });
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::PROP,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_prop(&prop).expect("encode prop"),
    });

    let path = tile_container_path(&world_layout, region, tile_id);
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let first = reader.directory.first().expect("directory entry");
    let second_offset = reader.header.section_dir_offset + DIR_ENTRY_SIZE as u64 + 12;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .expect("open tile");
    file.seek(SeekFrom::Start(second_offset))
        .expect("seek to offset field");
    file.write_all(&first.offset.to_le_bytes())
        .expect("overwrite offset");
    file.sync_all().expect("sync dir");

    let issues = world::validator::validate_project(temp.path());
    assert!(
        issues.iter().any(|issue| issue.message.contains("overlap")),
        "expected overlap issue"
    );
}

#[test]
fn tile_container_bounds_detected_by_validator() {
    let temp = tempdir().expect("tempdir");
    let project_manifest = world::schema::ProjectManifest::default();
    let project_layout = create_project(temp.path(), &project_manifest).expect("create project");
    let world_manifest = WorldManifest {
        world_id: "world_0".to_string(),
        regions: vec![RegionManifest {
            region_id: "region_0".to_string(),
            name: "Region 0".to_string(),
            bounds: RegionBounds::new(0, 0, 1, 1),
        }],
        ..WorldManifest::default()
    };
    let world_layout = create_world(&project_layout, &world_manifest).expect("create world");

    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 7, y: -3 },
    };
    let region_hash = hash_region(region);
    let spec_hash = hash_world_spec(DEFAULT_WORLD_SPEC);
    let header = TileContainerHeader::new(tile_id.coord.x, tile_id.coord.y, region_hash, spec_hash);

    let meta = MetaSection {
        format_version: WORLD_FORMAT_VERSION,
        tile_id,
        region_hash,
        created_timestamp: 0,
    };
    let prop = PropSection { instances: vec![] };

    let mut writer = TileContainerWriter::new().alignment(DEFAULT_ALIGNMENT);
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::META,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_meta(&meta),
    });
    writer.add_section(TileSectionPayload {
        tag: TileSectionTag::PROP,
        section_version: 1,
        codec: 0,
        flags: 0,
        decoded: encode_prop(&prop).expect("encode prop"),
    });

    let path = tile_container_path(&world_layout, region, tile_id);
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let entry_offset = reader.header.section_dir_offset + DIR_ENTRY_SIZE as u64 + 20;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .expect("open tile");
    file.seek(SeekFrom::Start(entry_offset))
        .expect("seek to stored_len field");
    file.write_all(&reader.file_len.to_le_bytes())
        .expect("overwrite stored_len");
    file.sync_all().expect("sync dir");

    let issues = world::validator::validate_project(temp.path());
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("out of bounds")),
        "expected bounds issue"
    );
}
