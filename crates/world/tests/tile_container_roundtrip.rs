use tempfile::tempdir;
use world::schema::WORLD_FORMAT_VERSION;
use world::tile_container::world_spec_hash::{hash_region, hash_world_spec, DEFAULT_WORLD_SPEC};
use world::tile_container::{
    decode_hmap, decode_meta, encode_hmap, encode_meta, HmapSection, MetaSection,
    TileContainerHeader, TileContainerReader, TileContainerWriter, TileSectionPayload,
    TileSectionTag, DEFAULT_ALIGNMENT,
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
