use crate::{
    parser,
    types::{BinaryComponentConfig, BuildType, Component, ComponentType, Config},
    utils::change_root,
};
use rayon::prelude::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub fn build(config: &Config, _iso: bool) -> io::Result<()> {
    let components = parser::read_components(&config.components_dir)?;

    components
        .par_iter()
        .filter(|component| component.component_type == ComponentType::Binary)
        .for_each(|component| build_component(component, &config.build_target, &config.rootfs_dir).expect("Failed to compile component"));

    Ok(())
}

fn build_component(component: &Component, target: &str, rootfs_path: &Path) -> io::Result<()> {
    println!("Building component: {}", component.name);

    if component.config.is_none() {
        panic!("Missing config for component: {}", component.name);
    }

    let config = toml::de::from_str::<BinaryComponentConfig>(&component.config.as_ref().unwrap().to_string())
        .expect("Failed to parse component config");

    let build_type_args = match config.build_type {
        BuildType::Debug => vec!["build"],
        BuildType::Release => vec!["build", "--release"],
    };

    Command::new("cargo")
        .args(build_type_args)
        .args(["--target", target])
        .current_dir(&component.path)
        .spawn()
        .expect("Failed to build with cargo")
        .wait()
        .expect("Failed to wait for cargo build");

    let build_out = component
        .path
        .join("target")
        .join(target)
        .join(match config.build_type {
            BuildType::Debug => "debug",
            BuildType::Release => "release",
        })
        .join(&component.name);

    assert!(build_out.exists(), "Failed to build component: {}", component.name);

    let binary_out = if config.out == PathBuf::from("/dev/null") {
        return Ok(());
    } else {
        change_root(&config.out, rootfs_path)
    };

    let binary_out_directory = binary_out.parent().unwrap();
    fs::create_dir_all(binary_out_directory).expect("Failed to create directory for binary");
    fs::copy(&build_out, &binary_out).expect("Failed to copy binary");

    Ok(())
}
