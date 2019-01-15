use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

bitflags! {
    pub flags FileType: u16 {
        const FILE      = 0b0001,
        const DIRECTORY = 0b0010,
        const ARCHIVE   = DIRECTORY.bits | FILE.bits,
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub is_readonly: bool,
    pub file_type: FileType,
    pub len: Option<u64>,
    pub created: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub modified: Option<SystemTime>,
}

impl From<fs::Metadata> for Metadata {
    fn from(meta: fs::Metadata) -> Self {
        Self {
            is_readonly: meta.permissions().readonly(),
            file_type: if meta.file_type().is_file() {
                FILE
            } else {
                DIRECTORY
            },
            len: Some(meta.len()),
            created: meta.created().ok(),
            accessed: meta.accessed().ok(),
            modified: meta.modified().ok(),
        }
    }
}

pub trait File: io::Read + io::Seek + io::Write {
    fn metadata(&self) -> io::Result<Metadata>;
}

bitflags! {
    pub flags OpenOptions: u16 {
        const READ       = 0b0000_0001,
        const WRITE      = 0b0000_0010,
        const APPEND     = 0b0000_0100 | WRITE.bits,
        const TRUNCATE   = 0b0000_1000 | WRITE.bits,
        const CREATE     = 0b0001_0000 | WRITE.bits,
        const CREATE_NEW = TRUNCATE.bits | WRITE.bits,
    }
}

pub trait Filesystem: Send + Sync {
    /* TODO where to add #[must_use]? */

    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    fn open_file(&self, path: &Path, opts: OpenOptions) -> io::Result<Box<File>>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;

    fn read_dir(&self, path: &Path) -> io::Result<BTreeMap<PathBuf, Metadata>>;
    fn create_dir(&self, path: &Path) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_dir(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;

    // TODO fn copy(&self, from: &Path, to: &Path)  -> io::Result<()>;
    // TODO fn move(&self, from: &Path, to: &Path)  -> io::Result<()>;
    // TODO fn rename(&self, from: &Path, to: &Path)  -> io::Result<()>;
}
