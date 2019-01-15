use filesystem::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

/// Corresponds to `HOME`.
pub const HOME_DIR: &'static str = "$home";

/// Corresponds to `XDG_DATA_HOME`.
pub const DATA_HOME_DIR: &'static str = "$data_home";

/// Corresponds to `XDG_CONFIG_HOME`.
pub const CONFIG_HOME_DIR: &'static str = "$config_home";

/// Corresponds to `XDG_CACHE_HOME`.
pub const CACHE_HOME_DIR: &'static str = "$cache_home";

/// Corresponds to the first path in `XDG_DATA_DIRS`.
pub const DATA_DIR: &'static str = "$data";

/// Corresponds to the first path in `XDG_CONFIG_DIRS`.
pub const CONFIG_DIR: &'static str = "$config";

// TODO
// Corresponds to `XDG_RUNTIME_DIR`.
// pub const RUNTIME_DIR: &'static str = "$runtime";

// TODO?
//pub const TEMP_DIR: &'static str = "$temp";
//pub const UNIQUE_TEMP_DIR: &'static str = "$unique_temp";

// TODO?
//pub const CURRENT_EXE_DIR: &'static str = "$current_exe_dir";
//pub const REAL_EXE_DIR: &'static str = "$real_exe_dir";

#[cfg(not(feature = "xdg"))]
fn xdg_path(
    prefix: Option<&Path>,
    profile: Option<&Path>,
    path: &Path,
) -> Option<Result<PathBuf, Box<Error>>> {
    None
}

#[cfg(feature = "xdg")]
fn xdg_path(
    prefix: Option<&Path>,
    profile: Option<&Path>,
    path: &Path,
) -> Result<Option<PathBuf>, Box<Error>> {
    use xdg::BaseDirectories;

    let xdg = match (prefix, profile) {
        (Some(prefix), Some(profile)) => BaseDirectories::with_profile(prefix, profile)?,
        (Some(prefix), None) => BaseDirectories::with_prefix(prefix)?,
        _ => BaseDirectories::new()?,
    };

    let (real_prefix, rest) = if let Ok(rest) = path.strip_prefix(DATA_HOME_DIR) {
        (xdg.get_data_home(), rest)
    } else if let Ok(rest) = path.strip_prefix(CONFIG_HOME_DIR) {
        (xdg.get_config_home(), rest)
    } else if let Ok(rest) = path.strip_prefix(CACHE_HOME_DIR) {
        (xdg.get_cache_home(), rest)
    } else if let Ok(rest) = path.strip_prefix(DATA_DIR) {
        (xdg.get_data_dirs().remove(0), rest)
    } else if let Ok(rest) = path.strip_prefix(CONFIG_DIR) {
        (xdg.get_config_dirs().remove(0), rest)
    } else {
        return Ok(None);
    };

    let mut ret = real_prefix;
    ret.push(rest);
    Ok(Some(ret))
}

fn canonicalize(
    prefix: Option<&Path>,
    profile: Option<&Path>,
    path: &Path,
) -> Result<PathBuf, Box<Error>> {
    super::validate_path(&path)?;

    if let Ok(rest) = path.strip_prefix(HOME_DIR) {
        let mut home_path = env::home_dir().unwrap();
        home_path.push(rest);
        Ok(fs::canonicalize(home_path)?)
    } else if let Some(xdg_path) = xdg_path(prefix, profile, path)? {
        Ok(fs::canonicalize(xdg_path)?)
    } else if path.starts_with("$") {
        unimplemented!() // TODO return error
    } else {
        Ok(fs::canonicalize(path)?)
    }
}

// ++++++++++++++++++++ StdFile ++++++++++++++++++++

pub struct StdFile(fs::File);

impl From<fs::File> for StdFile {
    fn from(file: fs::File) -> Self { StdFile(file) }
}

// TODO?
//impl Drop for StdFile {
//    fn drop(&mut self){ self.0.sync_all();  }
//}

impl io::Read for StdFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.0.read(buf) }
}

impl io::Seek for StdFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> { self.0.seek(pos) }
}

impl io::Write for StdFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.0.write(buf) }
    fn flush(&mut self) -> io::Result<()> { self.0.flush() }
}

impl File for StdFile {
    fn metadata(&self) -> io::Result<Metadata> { self.0.metadata().map(|meta| meta.into()) }
}

// ++++++++++++++++++++ StdFs ++++++++++++++++++++

/// TODO Naming? OsFs?
pub struct StdFs {
    base: PathBuf,
    // TODO readonly: bool,
}

impl StdFs {
    fn _new(base: PathBuf) -> Result<Self, Box<Error>> {
        // TODO ensure base exists & is dir
        Ok(Self { base })
    }
    pub fn new<P>(base: P) -> Result<Self, Box<Error>>
    where
        P: AsRef<Path>,
    {
        Self::_new(canonicalize(None, None, base.as_ref())?)
    }
    pub fn with_prefix<P1, P2>(prefix: P1, base: P2) -> Result<Self, Box<Error>>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        Self::_new(canonicalize(Some(prefix.as_ref()), None, base.as_ref())?)
    }
    pub fn with_profile<P1, P2, P3>(
        prefix: P1,
        profile: P2,
        base: P3,
    ) -> Result<Self, Box<Error>>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        Self::_new(canonicalize(
            Some(prefix.as_ref()),
            Some(profile.as_ref()),
            base.as_ref(),
        )?)
    }
}

impl Filesystem for StdFs {
    fn metadata(&self, vpath: &Path) -> io::Result<Metadata> {
        let _ = super::validate_path(vpath)?;

        fs::metadata(self.base.join(vpath)).map(|meta| meta.into())
    }
    fn open_file(&self, vpath: &Path, opts: OpenOptions) -> io::Result<Box<File>> {
        let _ = super::validate_path(vpath)?;

        let file = fs::OpenOptions::new()
            .read(opts.contains(READ))
            .write(opts.contains(WRITE))
            .append(opts.contains(APPEND))
            .truncate(opts.contains(TRUNCATE))
            .create(opts.contains(CREATE))
            .create_new(opts.contains(CREATE_NEW))
            .open(self.base.join(vpath))?;
        Ok(Box::new(StdFile(file)))
    }
    fn remove_file(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        fs::remove_file(self.base.join(vpath))
    }
    fn read_dir(&self, vpath: &Path) -> io::Result<BTreeMap<PathBuf, Metadata>> {
        let _ = super::validate_path(vpath)?;

        let mut ret = BTreeMap::new();
        for entry in fs::read_dir(self.base.join(vpath))? {
            let entry = entry?;
            let vpath = entry.path().strip_prefix(&self.base).unwrap().to_owned();
            let meta = Metadata::from(entry.metadata()?);
            ret.insert(vpath, meta);
        }
        Ok(ret)
    }
    fn create_dir(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        fs::create_dir(self.base.join(vpath))
    }
    fn create_dir_all(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        fs::create_dir_all(self.base.join(vpath))
    }
    fn remove_dir(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        fs::remove_dir(self.base.join(vpath))
    }
    fn remove_dir_all(&self, vpath: &Path) -> io::Result<()> {
        let _ = super::validate_path(vpath)?;

        fs::remove_dir_all(self.base.join(vpath))
    }
}
