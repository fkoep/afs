use filesystem::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

// ++++++++++++++++++++ MountError ++++++++++++++++++++

/// TODO? Into<io::Error>
#[derive(Debug)]
pub enum MountError {
    /// See `::validate_path()`.
    InvalidPath,

    /// Locations must have distinct paths. No path may be a sub-path of
    /// another.
    LocationOverlap,
}

impl fmt::Display for MountError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        fmt::Debug::fmt(self, fmt)
    }
}

impl Error for MountError {
    fn description(&self) -> &str {
        match self {
            &MountError::InvalidPath => "Mount error: Invalid path",
            &MountError::LocationOverlap => "Mount error: Location overlap",
        }
    }
}

// ++++++++++++++++++++ VirtualFs ++++++++++++++++++++

#[derive(Default)]
pub struct VirtualFs {
    mounted: BTreeMap<PathBuf, Box<Filesystem>>,
}

impl VirtualFs {
    pub fn new() -> Self { Self::default() }

    #[must_use]
    pub fn mount<P, F>(&mut self, vpath: P, fs: F) -> Result<(), MountError>
    where
        P: Into<PathBuf>,
        F: Filesystem + 'static,
    {
        let vpath = vpath.into();
        if !super::is_valid_path(&vpath) {
            return Err(MountError::InvalidPath);
        }

        if self
            .mounted
            .keys()
            .any(|vbase| vbase.starts_with(&vpath) || vpath.starts_with(vbase))
        {
            return Err(MountError::LocationOverlap);
        }

        self.mounted.insert(vpath, Box::new(fs));
        Ok(())
    }

    pub fn unmount<P>(&mut self, vpath: &P) -> Option<Box<Filesystem>>
    where
        P: AsRef<Path>,
    {
        self.mounted.remove(vpath.as_ref())
    }

    pub fn unmount_all(&mut self) { self.mounted.clear() }
}

const VDIR_META: Metadata = Metadata {
    is_readonly: true,
    file_type: DIRECTORY,
    len: None,
    created: None,
    accessed: None,
    modified: None,
};

// TODO include path
fn not_found<R>() -> io::Result<R> {
    Err(io::Error::new(io::ErrorKind::NotFound, "")) // TODO? errmsg
}

// TODO include path
fn permission_denied<R>() -> io::Result<R> {
    // "virtual directories can only be modified through mounting & unmounting"
    Err(io::Error::new(io::ErrorKind::PermissionDenied, "")) // TODO? errmsg
}

impl Filesystem for VirtualFs {
    fn metadata(&self, vpath: &Path) -> io::Result<Metadata> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.metadata(vrest);
            } else if vbase.starts_with(vpath) {
                return Ok(VDIR_META);
            }
        }
        not_found()
    }
    fn open_file(&self, vpath: &Path, opts: OpenOptions) -> io::Result<Box<File>> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.open_file(vrest, opts);
            } else if vbase.starts_with(vpath) {
                return permission_denied();
            }
        }
        not_found()
    }
    fn remove_file(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.remove_file(vrest);
            } else if vbase.starts_with(vpath) {
                return permission_denied();
            }
        }
        not_found()
    }
    fn read_dir(&self, vpath: &Path) -> io::Result<BTreeMap<PathBuf, Metadata>> {
        let _ = super::validate_path(vpath)?;

        let mut ret = BTreeMap::new();
        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                debug_assert!(ret.is_empty());
                return fs.read_dir(vrest);
            } else if let Ok(vdir) = vbase.strip_prefix(vpath) {
                ret.insert(vdir.to_owned(), VDIR_META);
            }
        }
        if !ret.is_empty() {
            Ok(ret)
        } else {
            not_found()
        }
    }
    fn create_dir(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.create_dir(vrest);
            }
        }
        permission_denied()
    }
    fn create_dir_all(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.create_dir_all(vrest);
            }
        }
        permission_denied()
    }
    fn remove_dir(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.remove_dir(vrest);
            } else if vbase.starts_with(vpath) {
                return permission_denied();
            }
        }
        not_found()
    }
    fn remove_dir_all(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        for (vbase, fs) in &self.mounted {
            if let Ok(vrest) = vpath.strip_prefix(vbase) {
                return fs.remove_dir_all(vrest);
            } else if vbase.starts_with(vpath) {
                return permission_denied();
            }
        }
        not_found()
    }
}
