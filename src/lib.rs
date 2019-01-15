// ++++++++++++++++++++ private dependencies ++++++++++++++++++++

#[macro_use]
extern crate bitflags;
extern crate lazy_static;
extern crate tempdir;
#[cfg(feature = "xdg")]
extern crate xdg;

// ++++++++++++++++++++ submodules ++++++++++++++++++++

mod filesystem;
mod std_fs;
mod virtual_fs;

pub use filesystem::*;
pub use std_fs::*;
pub use virtual_fs::*;

// ++++++++++++++++++++ lib.rs ++++++++++++++++++++

use std::io;
use std::path::{Component, Path};

/// Returns `true` if a path fits `myfs`'s restrictions and may be used.
pub fn is_valid_path(path: &Path) -> bool {
    path.components().all(|c| {
        if let Component::Normal(_) = c {
            true
        } else {
            false
        }
    })
}

/// Checks whether a path fits `myfs`'s restrictions and may be used.
///
/// Returns `Ok(())` or an `InvalidInput` error.
pub fn validate_path(path: &Path) -> io::Result<()> {
    let errmsg = "myfs only allows the use of normal path components \
                  (no windows prefix, no root dir, no current dir, no parent dir)";

    if !is_valid_path(path) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, errmsg));
    }
    Ok(())
}
