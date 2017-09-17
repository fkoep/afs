// ++++++++++++++++++++ private dependencies ++++++++++++++++++++

#[macro_use]
extern crate bitflags;
extern crate tempdir;
extern crate lazy_static;
#[cfg(feature = "xdg")]
extern crate xdg;

// ++++++++++++++++++++ submodules ++++++++++++++++++++

mod filesystem;
mod virtual_fs;
mod std_fs;

pub use filesystem::*;
pub use virtual_fs::*;
pub use std_fs::*;

// ++++++++++++++++++++ lib.rs ++++++++++++++++++++

use std::path::{Component, Path};
use std::io;

/// Returns `true` if a path fits `myfs`'s restrictions and may be used.
pub fn is_valid_path(path: &Path) -> bool {
    path.components()
        .all(|c| if let Component::Normal(_) = c { true } else { false })
}

/// Checks whether a path fits `myfs`'s restrictions and may be used.
///
/// Returns `Ok(())` or an `InvalidInput` error.
pub fn validate_path(path: &Path) -> io::Result<()> {
	let errmsg = "myfs only allows the use of normal path components \
                  (no windows prefix, no root dir, no current dir, no parent dir)";

    if !is_valid_path(path) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, errmsg))
	}
	Ok(())
}
