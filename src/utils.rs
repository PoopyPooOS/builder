#![allow(clippy::needless_pass_by_value)]

use std::path::{Path, PathBuf};

pub fn change_root(path: &Path, new_root: &Path) -> PathBuf {
    let relative_path = path.strip_prefix("/").unwrap();
    new_root.join(relative_path)
}
