use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

use tempfile::tempdir;
use world::schema::WORLD_FORMAT_VERSION;
use world::tile_container::world_spec_hash::{hash_region, hash_world_spec, DEFAULT_WORLD_SPEC};
use world::tile_container::{
    decode_meta, encode_hmap, encode_meta, HmapSection, MetaSection, TileContainerHeader,
    TileContainerReader, TileContainerWriter, TileSectionPayload, TileSectionTag,
    DEFAULT_ALIGNMENT, DIR_ENTRY_SIZE,
};
use world::{TileCoord, TileId};

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
fn tile_container_reader_rejects_out_of_bounds_section() {
    let temp = tempdir().expect("tempdir");
    let region = "region_0";
    let tile_id = TileId {
        coord: TileCoord { x: 0, y: 1 },
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

    let path = temp.path().join("x0_y1.tile");
    writer.write(&path, header).expect("write tile");

    let reader = TileContainerReader::open(&path).expect("read tile");
    let index = reader
        .directory
        .iter()
        .position(|entry| entry.tag == TileSectionTag::HMAP)
        .expect("HMAP entry");
    let entry_offset = reader.header.section_dir_offset + index as u64 * DIR_ENTRY_SIZE as u64 + 20;
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

    let reader = TileContainerReader::open(&path).expect("read tile");
    assert!(
        reader.read_section(TileSectionTag::HMAP).is_err(),
        "expected bounds failure"
    );
}
