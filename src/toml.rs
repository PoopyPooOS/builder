use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub fn parse<T: DeserializeOwned>(path: impl Into<PathBuf>) -> T {
    let raw = std::fs::read_to_string(path.into()).expect("Failed to read toml file");
    toml::from_str::<T>(&raw).expect("Failed to parse toml file")
}
