use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn change_root(path: &Path, new_root: &Path) -> PathBuf {
    let relative_path = path.strip_prefix("/").unwrap();
    new_root.join(relative_path)
}

pub fn parse<T: for<'a> Deserialize<'a>>(path: impl Into<PathBuf>) -> T {
    let raw = fs::read_to_string(path.into()).expect("Failed to read toml file");
    toml::from_str::<T>(&raw).expect("Failed to parse toml file")
}
