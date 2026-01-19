use crate::tile_container::{
    TileContainerHeader, TileSectionDirEntry, TileSectionTag, CONTAINER_VERSION, DIR_ENTRY_SIZE,
    HEADER_SIZE, MAX_SECTION_COUNT, MIN_CONTAINER_VERSION,
};
use anyhow::{anyhow, bail, Context};
use crc32fast::Hasher;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TileContainerReader {
    pub path: PathBuf,
    pub header: TileContainerHeader,
    pub directory: Vec<TileSectionDirEntry>,
    pub file_len: u64,
}

impl TileContainerReader {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path).with_context(|| format!("open tile {:?}", path))?;
        let file_len = file.metadata()?.len();
        if file_len < HEADER_SIZE as u64 {
            bail!("tile too small");
        }

        let mut header_bytes = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_bytes)?;
        let header = TileContainerHeader::from_bytes(&header_bytes)?;

        if header.container_version < MIN_CONTAINER_VERSION {
            bail!(
                "container version {} below minimum {}",
                header.container_version,
                MIN_CONTAINER_VERSION
            );
        }
        if header.container_version > CONTAINER_VERSION {
            bail!(
                "container version {} exceeds supported {}",
                header.container_version,
                CONTAINER_VERSION
            );
        }

        if header.section_count > MAX_SECTION_COUNT {
            bail!("section_count {} exceeds cap", header.section_count);
        }

        let dir_bytes_len = header.section_count as u64 * DIR_ENTRY_SIZE as u64;
        let dir_end = header.section_dir_offset + dir_bytes_len;
        if dir_end > file_len {
            return Err(anyhow!("section directory exceeds file bounds"));
        }

        file.seek(SeekFrom::Start(header.section_dir_offset))?;
        let mut dir_bytes = vec![0u8; dir_bytes_len as usize];
        file.read_exact(&mut dir_bytes)?;

        let mut directory = Vec::with_capacity(header.section_count as usize);
        for chunk in dir_bytes.chunks_exact(DIR_ENTRY_SIZE) {
            directory.push(TileSectionDirEntry::from_bytes(chunk)?);
        }

        Ok(Self {
            path,
            header,
            directory,
            file_len,
        })
    }

    pub fn section(&self, tag: TileSectionTag) -> Option<&TileSectionDirEntry> {
        self.directory.iter().find(|entry| entry.tag == tag)
    }

    pub fn read_section(&self, tag: TileSectionTag) -> anyhow::Result<Vec<u8>> {
        let entry = self
            .section(tag)
            .ok_or_else(|| anyhow!("section {} not found", tag))?;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let mut buffer = vec![0u8; entry.stored_len as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_section_checked(&self, tag: TileSectionTag) -> anyhow::Result<Vec<u8>> {
        let entry = self
            .section(tag)
            .ok_or_else(|| anyhow!("section {} not found", tag))?;
        let buffer = self.read_section(tag)?;
        let mut hasher = Hasher::new();
        hasher.update(&buffer);
        let crc = hasher.finalize();
        if crc != entry.crc32 {
            bail!("crc mismatch for section {}", tag);
        }
        Ok(buffer)
    }

    pub fn decode_section(&self, tag: TileSectionTag) -> anyhow::Result<Vec<u8>> {
        let entry = self
            .section(tag)
            .ok_or_else(|| anyhow!("section {} not found", tag))?;
        if entry.codec != 0 {
            bail!("unsupported codec {} for section {}", entry.codec, tag);
        }
        self.read_section_checked(tag)
    }
}
