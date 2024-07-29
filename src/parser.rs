use crate::types::{BinaryComponentConfig, Component, ComponentType};
use serde::de::DeserializeOwned;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn read_components(path: &Path) -> io::Result<Vec<Component>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| entry.ok().and_then(|e| if e.file_type().ok()?.is_dir() { Some(e) } else { None }))
        .filter(|entry| {
            !entry
                .file_name()
                .into_string()
                .expect("Failed to get component name")
                .starts_with(".")
        })
        .map(|component| {
            let component_type: ComponentType = match &component {
                e if e.path().join("build.toml").exists() => {
                    ComponentType::Binary(parse::<BinaryComponentConfig>(e.path().join("build.toml")))
                }
                e if e.path().join("module.toml").exists() => ComponentType::Module,
                _ => ComponentType::Other,
            };

            Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type,
            }
        })
        .collect::<Vec<_>>())
}

pub fn parse<T: DeserializeOwned>(path: impl Into<PathBuf>) -> T {
    let raw = std::fs::read_to_string(path.into()).expect("Failed to read toml file");
    toml::from_str::<T>(&raw).expect("Failed to parse toml file")
}
