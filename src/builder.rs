use crate::{
    parser,
    types::{BinaryComponentConfig, BuildType, Component, ComponentType, Config},
    utils::change_root,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    sync::Arc,
    time::Duration,
};

pub fn build(config: &Config, _iso: bool) -> io::Result<()> {
    let components = parser::read_components(&config.components_dir)?;
    let mpb = Arc::new(MultiProgress::new());

    components
        .par_iter()
        .filter(|component| component.component_type == ComponentType::Binary)
        .for_each(|component| {
            build_component(&mpb, component, &config.build_target, &config.rootfs_dir).expect("Failed to compile component")
        });

    mpb.clear().expect("Failed to clear progress bars");

    // TODO: Build ISO

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} {job_name}: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_message("Building initrd");
    let initrd = Command::new("sh")
        .args([
            "-c",
            &format!("find . | cpio -o -H newc > {}", config.dist_dir.join("iso/boot/initrd").display()),
        ])
        .current_dir(&config.rootfs_dir)
        .spawn()
        .expect("Failed to create initrd")
        .wait()
        .expect("Failed to wait for initrd");

    if initrd.success() {
        pb.finish_with_message("Finished building initrd");
    } else {
        pb.finish_with_message("Failed to build initrd");
        process::exit(1);
    }

    drop(pb);

    Ok(())
}

fn build_component(mpb: &Arc<MultiProgress>, component: &Component, target: &str, rootfs_path: &Path) -> io::Result<()> {
    if component.config.is_none() {
        panic!("Missing config for component: {}", component.name);
    }

    let config = toml::de::from_str::<BinaryComponentConfig>(&component.config.as_ref().unwrap().to_string())
        .expect("Failed to parse component config");

    compile(&mpb, &component.name, &component.path, &config.build_type, target)?;

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

fn compile(mpb: &Arc<MultiProgress>, job_name: &str, workspace_path: &Path, build_type: &BuildType, target: &str) -> io::Result<()> {
    let build_type_args = match build_type {
        BuildType::Debug => vec!["build"],
        BuildType::Release => vec!["build", "--release"],
    };

    let pb = mpb.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} {job_name}: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );

    let mut cargo = Command::new("cargo")
        .args(build_type_args)
        .args(["--target", target])
        .stderr(Stdio::piped())
        .current_dir(workspace_path)
        .spawn()
        .expect("Failed to wait for cargo build");

    for line in BufReader::new(cargo.stderr.take().unwrap()).lines() {
        let line = line.unwrap().trim().to_string();

        if !line.is_empty() {
            pb.set_message(format!("{}: {}", job_name, line));
        }
    }

    mpb.add(pb);

    cargo.wait().unwrap();

    Ok(())
}
