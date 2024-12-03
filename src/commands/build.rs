use crate::{
    config::Config,
    parser::{self, BuildType, Component, ComponentConfig, ComponentType},
    utils::change_root,
};
use log::{error, info};
use std::{
    fs,
    path::Path,
    process::{self, Command, Stdio},
};
use tokio::io;

pub async fn build(config: &Config, build_iso: bool) -> io::Result<()> {
    info!("Building project");
    let components = parser::read_components(&config.builder.components_dir)?;

    // Build all the components and copy them to the rootfs.
    for component in components {
        if !component.component_type.is_binary() {
            continue;
        }

        info!("Building component: {}", component.name);
        build_component(&component, &config.builder.build_target, &config.builder.rootfs_dir)?;
    }

    // Build initrd
    info!("Building initrd");
    let mut initrd = Command::new("sh");
    initrd.args([
        "-c",
        &format!(
            "find . | cpio -o -H newc > {}",
            config.builder.dist_dir.join("iso/boot/initrd").display()
        ),
    ]);
    initrd.stderr(Stdio::null());
    initrd.current_dir(&config.builder.rootfs_dir);

    let initrd = match initrd.spawn() {
        Ok(mut child) => child.wait().unwrap(),
        Err(err) => {
            error!("Failed to build initrd: {err}");
            process::exit(1);
        }
    };

    if initrd.success() {
        info!("Initrd built successfully");
    } else {
        error!("Failed to build initrd");
        process::exit(1);
    }

    if !build_iso {
        return Ok(());
    }

    // Build ISO
    let mut iso = Command::new("grub-mkrescue");
    iso.args(["-o", &config.builder.dist_dir.join("PoopyPooOS.iso").display().to_string()]);
    iso.arg(config.builder.dist_dir.join("iso").display().to_string());
    iso.stderr(Stdio::null());
    iso.current_dir(&config.builder.dist_dir);

    let iso = match iso.spawn() {
        Ok(mut child) => child.wait().unwrap(),
        Err(err) => {
            error!("Failed to build ISO: {err}");
            process::exit(1);
        }
    };

    if iso.success() {
        info!("ISO built successfully");
    } else {
        error!("Failed to build ISO");
        process::exit(1);
    }

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn build_component(component: &Component, target: impl Into<String>, rootfs_path: impl AsRef<Path>) -> io::Result<()> {
    let target: String = target.into();
    let rootfs_path = rootfs_path.as_ref();

    let ComponentType::Binary(config) = &component.component_type else {
        unreachable!()
    };

    let target = if let Some(target) = &config.build_target {
        target
    } else {
        &target.to_string()
    };

    compile(&component.path, config, target);

    let build_out = component
        .path
        .join("target")
        .join(target)
        .join(config.build_type.to_string())
        .join(&component.name);

    if !build_out.exists() {
        error!("Failed to build component: {}", component.name);
        process::exit(1);
    }

    let binary_out = change_root(&config.out, rootfs_path);
    let binary_out_directory = binary_out.parent().unwrap();
    let _ = fs::create_dir_all(binary_out_directory);
    match fs::copy(build_out, binary_out) {
        Ok(_) => (),
        Err(err) => {
            error!("Failed to copy component build output: {err}");
            process::exit(1);
        }
    }

    if let Some(post_copy_script) = &config.post_copy_script {
        let post_copy_script = format!(
            "
            ROOTFS={}
            OUT={}
            {}",
            rootfs_path.display(),
            config.out.display(),
            post_copy_script
        );

        let mut sh = Command::new("sh");
        sh.arg("-c");
        sh.arg(post_copy_script.trim());

        match sh.spawn() {
            Ok(mut child) => child.wait().unwrap(),
            Err(err) => {
                error!("Failed to run post copy script: {err}");
                process::exit(1);
            }
        };
    }

    Ok(())
}

fn compile(workspace_path: &Path, config: &ComponentConfig, target: &str) {
    let build_type_args = match config.build_type {
        BuildType::Debug => vec!["build"],
        BuildType::Release => vec!["build", "--release"],
    };

    let mut cargo = Command::new("cargo");
    cargo.args(build_type_args);
    cargo.args(["--target", target]);
    cargo.current_dir(workspace_path);

    match cargo.spawn() {
        Ok(mut child) => child.wait().unwrap(),
        Err(err) => {
            error!("Failed to compile component: {err}");
            process::exit(1);
        }
    };
}
