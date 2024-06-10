use serde::Deserialize;
use std::{
    env::current_dir,
    fs::{self, DirEntry},
    path::PathBuf,
    process::Command,
};

use crate::Config;

#[derive(Debug, Clone, Deserialize)]
pub enum BuildType {
    Debug,
    Release,
}

#[derive(Deserialize)]
struct ComponentConfig {
    out: String,
    binary_path: Option<String>,
}

pub fn build(name: &str, config: &Config) {
    let cwd = current_dir().expect("Failed to get current directory");
    let component_path = {
        let mut ancestors = cwd.ancestors();
        ancestors.next();
        let path = ancestors.next().unwrap();
        path.join(&config.components_dir).join(name)
    };

    let is_module = component_path.join("module.toml").exists();

    if is_module {
        return build_module(component_path, config);
    }

    let build_args = match config.build_type {
        BuildType::Debug => vec!["build"],
        BuildType::Release => vec!["build", "--release"],
    };

    Command::new("cargo")
        .args(build_args)
        .current_dir(&component_path)
        .output()
        .expect("Failed to execute 'cargo build'");

    let build_config = {
        let raw_toml = fs::read_to_string(component_path.join("build.toml")).expect("Failed to read build config");
        let parsed: ComponentConfig = toml::from_str(&raw_toml).expect("Failed to parse build config");

        parsed
    };

    let component_binary_path: PathBuf = if build_config.binary_path.is_none() {
        match config.build_type {
            BuildType::Debug => component_path.join(format!("target/debug/{}", name)),
            BuildType::Release => component_path.join(format!("target/release/{}", name)),
        }
    } else {
        change_root(PathBuf::from(&build_config.binary_path.unwrap()), component_path)
    };

    // Some components might use /dev/null as their output which means they should not be copied.
    let binary_out = if build_config.out == "/dev/null" {
        return;
    } else {
        change_root(PathBuf::from(&build_config.out), PathBuf::from(&config.rootfs_dir))
    };

    let binary_out_directory = binary_out.parent().unwrap();

    fs::create_dir_all(binary_out_directory).unwrap_or_else(|_| panic!("Failed to create parent directories for component {}", name));

    fs::copy(&component_binary_path, &binary_out)
        .unwrap_or_else(|_| panic!("Failed to copy {} to {}", component_binary_path.display(), binary_out.display()));
}

fn build_module(module_path: PathBuf, config: &Config) {
    let module_components: Vec<DirEntry> = fs::read_dir(&module_path)
        .expect("Failed to read module directory")
        .filter_map(Result::ok)
        .filter(|file| file.file_type().unwrap().is_dir())
        .collect();

    let mut new_config = config.clone();
    new_config.components_dir = module_path.display().to_string();

    for component in module_components {
        let component_name = component.file_name().to_str().unwrap().to_string();
        build(&component_name, &new_config);
    }
}

pub fn change_root(path: PathBuf, new_root: PathBuf) -> PathBuf {
    let relative_path = path.strip_prefix("/").unwrap();
    new_root.join(relative_path)
}
