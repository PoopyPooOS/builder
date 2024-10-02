use crate::types::{Component, ComponentType};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

fn single_read_components(path: &Path) -> io::Result<Vec<Component>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| entry.ok().and_then(|e| if e.file_type().ok()?.is_dir() { Some(e) } else { None }))
        .filter(|entry| {
            !entry
                .file_name()
                .into_string()
                .expect("Failed to get component name")
                .starts_with('.')
        })
        .map(|component| match &component {
            e if e.path().join("build.toml").exists() => Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type: ComponentType::Binary,

                config: Some(parse::<toml::Table>(e.path().join("build.toml"))),
            },
            e if e.path().join("module.toml").exists() => Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type: ComponentType::Module,

                config: None,
            },
            _ => Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type: ComponentType::Other,

                config: None,
            },
        })
        .collect::<Vec<_>>())
}

fn get_component_name(component: &Component) -> Option<String> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&component.path)
        .output()
        .expect("Failed to execute cargo metadata");

    let metadata: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse JSON");

    metadata.get("packages").and_then(|p| p.as_array()).map(|packages| {
        packages[0]
            .get("name")
            .expect("Failed to get component name")
            .as_str()
            .unwrap()
            .to_string()
    })
}

pub fn read_components(path: &Path) -> io::Result<Vec<Component>> {
    let mut components = single_read_components(path)?;

    components = components
        .into_iter()
        .flat_map(|component| {
            if component.component_type == ComponentType::Module {
                single_read_components(&component.path).unwrap_or_else(|_| Vec::new())
            } else {
                vec![component]
            }
        })
        .collect();

    for component in &mut components {
        if let Some(name) = get_component_name(component) {
            component.name = name;
        }
    }

    Ok(components)
}

pub fn parse<T: DeserializeOwned>(path: impl Into<PathBuf>) -> T {
    let raw = fs::read_to_string(path.into()).expect("Failed to read toml file");
    toml::from_str::<T>(&raw).expect("Failed to parse toml file")
}
