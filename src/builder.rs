use serde::Deserialize;
use std::{env::current_dir, fs, path::PathBuf, process::Command};

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

pub fn build(name: &str, config: Config) {
    println!("Building {}", name);
    let cwd = current_dir().expect("Failed to get current directory");
    let component_path = {
        let mut ancestors = cwd.ancestors();
        ancestors.next();
        let path = ancestors.next().unwrap();
        path.join(config.components_dir).join(name)
    };

    if component_path.is_file() {
        return;
    }

    let build_args = match config.build_type {
        BuildType::Debug => vec!["build"],
        BuildType::Release => vec!["build", "--release"],
    };

    let output = Command::new("/usr/bin/cargo")
        .args(build_args)
        .current_dir(&component_path)
        .output()
        .expect("Failed to execute 'cargo build'");

    if output.status.success() {
        println!("Binaries for {} built successfully", name);
    } else {
        println!("Binaries for {} failed to build", name);
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            println!("Build error output: {}", stderr);
        }
    }

    let build_config = {
        let raw_toml = fs::read_to_string(component_path.join("build.toml"))
            .expect("Failed to read build config");
        let parsed: ComponentConfig =
            toml::from_str(&raw_toml).expect("Failed to parse build config");
        parsed
    };

    let component_binary_path: PathBuf = if build_config.binary_path.is_none() {
        match config.build_type {
            BuildType::Debug => component_path.join(format!("target/debug/{}", name)),
            BuildType::Release => component_path.join(format!("target/release/{}", name)),
        }
    } else {
        PathBuf::from(build_config.binary_path.unwrap())
    };

    let binary_out = change_root(
        PathBuf::from(&build_config.out),
        PathBuf::from(&config.rootfs_dir),
    );

    let binary_out_directory = binary_out.parent().unwrap();

    fs::create_dir_all(binary_out_directory)
        .unwrap_or_else(|_| panic!("Failed to create parent directories for component {}", name));

    fs::copy(component_binary_path, build_config.out)
        .unwrap_or_else(|_| panic!("Failed to copy binary for component {}", name));

    println!("Finished building {}", name);
}

pub fn change_root(path: PathBuf, new_root: PathBuf) -> PathBuf {
    let relative_path = path.strip_prefix("/").unwrap();
    new_root.join(relative_path)
}
