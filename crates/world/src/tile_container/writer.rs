use crate::tile_container::{
    alignment_padding, TileContainerHeader, TileSectionDirEntry, TileSectionTag, CONTAINER_VERSION,
    DEFAULT_ALIGNMENT, DIR_ENTRY_SIZE, HEADER_SIZE,
};
use anyhow::{anyhow, Context};
use crc32fast::Hasher;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TileSectionPayload {
    pub tag: TileSectionTag,
    pub section_version: u16,
    pub codec: u16,
    pub flags: u32,
    pub decoded: Vec<u8>,
}

#[derive(Debug)]
pub struct TileContainerWriter {
    sections: Vec<TileSectionPayload>,
    alignment: u64,
}

impl TileContainerWriter {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            alignment: DEFAULT_ALIGNMENT,
        }
    }

    pub fn alignment(mut self, alignment: u64) -> Self {
        self.alignment = alignment.max(1);
        self
    }

    pub fn add_section(&mut self, payload: TileSectionPayload) {
        self.sections.push(payload);
    }

    pub fn write(
        &self,
        path: impl AsRef<Path>,
        mut header: TileContainerHeader,
    ) -> anyhow::Result<PathBuf> {
        if header.container_version != CONTAINER_VERSION {
            return Err(anyhow!("unsupported container version"));
        }

        let mut sections = self.sections.clone();
        sections.sort_by(|a, b| compare_sections(a.tag, b.tag));

        header.section_count = sections.len() as u32;
        header.section_dir_offset = HEADER_SIZE as u64;

        let dir_bytes_len = sections.len() as u64 * DIR_ENTRY_SIZE as u64;
        let mut current_offset = header.section_dir_offset + dir_bytes_len;

        let mut directory = Vec::with_capacity(sections.len());
        let mut stored_payloads = Vec::with_capacity(sections.len());

        for payload in sections {
            let padding = alignment_padding(current_offset, self.alignment);
            current_offset += padding;
            let stored_len = payload.decoded.len() as u64;
            let decoded_len = stored_len;
            let mut hasher = Hasher::new();
            hasher.update(&payload.decoded);
            let crc32 = hasher.finalize();

            directory.push(TileSectionDirEntry {
                tag: payload.tag,
                section_version: payload.section_version,
                codec: payload.codec,
                flags: payload.flags,
                offset: current_offset,
                stored_len,
                decoded_len,
                crc32,
            });

            stored_payloads.push(payload.decoded);
            current_offset += stored_len;
        }

        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create tile directory {:?}", parent))?;
        }
        let tmp_path = path.with_extension("tile.tmp");
        let mut file = File::create(&tmp_path)?;
        file.write_all(&header.to_bytes())?;

        for entry in &directory {
            file.write_all(&entry.to_bytes())?;
        }

        let mut payload_offset = header.section_dir_offset + dir_bytes_len;
        for payload in stored_payloads {
            let padding = alignment_padding(payload_offset, self.alignment);
            if padding > 0 {
                file.seek(SeekFrom::Current(padding as i64))?;
            }
            payload_offset += padding;
            file.write_all(&payload)?;
            payload_offset += payload.len() as u64;
        }

        file.sync_all()?;

        if path.exists() {
            let backup_path = path.with_extension("tile.bak");
            if backup_path.exists() {
                fs::remove_file(&backup_path)
                    .with_context(|| format!("remove backup {:?}", backup_path))?;
            }
            fs::rename(&path, &backup_path)
                .with_context(|| format!("backup existing tile {:?}", path))?;
        }

        fs::rename(&tmp_path, &path)
            .with_context(|| format!("rename temp tile {:?} -> {:?}", tmp_path, path))?;

        Ok(path)
    }
}

fn compare_sections(a: TileSectionTag, b: TileSectionTag) -> Ordering {
    canonical_rank(a)
        .cmp(&canonical_rank(b))
        .then_with(|| a.0.cmp(&b.0))
}

fn canonical_rank(tag: TileSectionTag) -> u8 {
    if tag == TileSectionTag::META {
        0
    } else if tag == TileSectionTag::HMAP {
        1
    } else if tag == TileSectionTag::WMAP {
        2
    } else if tag == TileSectionTag::LIQD {
        3
    } else if tag == TileSectionTag::PROP {
        4
    } else if tag == TileSectionTag::SPLN {
        5
    } else if tag == TileSectionTag::ADDX {
        6
    } else {
        255
    }
}
