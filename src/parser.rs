use crate::utils::parse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub path: PathBuf,
    #[allow(clippy::struct_field_names)]
    pub component_type: ComponentType,
}

#[derive(Debug)]
pub enum ComponentType {
    Binary(ComponentConfig),
    Module,
    Other,
}

impl PartialEq for ComponentType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ComponentType::Binary(_), ComponentType::Binary(_))
                | (ComponentType::Module, ComponentType::Module)
                | (ComponentType::Other, ComponentType::Other)
        )
    }
}

impl ComponentType {
    // Function to check if the component is of type `Binary`
    pub fn is_binary(&self) -> bool {
        matches!(self, ComponentType::Binary(_))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub out: PathBuf,
    #[serde(default)]
    pub build_type: BuildType,
    pub build_target: Option<String>,
    pub post_copy_script: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum BuildType {
    Debug,
    #[default]
    Release,
}

impl Display for BuildType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildType::Debug => write!(f, "debug"),
            BuildType::Release => write!(f, "release"),
        }
    }
}

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
                component_type: ComponentType::Binary(parse::<ComponentConfig>(e.path().join("build.toml"))),
            },
            e if e.path().join("module.toml").exists() => Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type: ComponentType::Module,
            },
            _ => Component {
                name: component.file_name().into_string().expect("Failed to get component name"),
                path: component.path(),
                component_type: ComponentType::Other,
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
        if component.component_type.is_binary()
            && let Some(name) = get_component_name(component)
        {
            component.name = name;
        }
    }

    Ok(components)
}
