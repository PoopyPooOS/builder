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

pub fn build(config: &Config, iso: bool) -> io::Result<()> {
    let components = parser::read_components(&config.builder.components_dir)?;
    let mpb = Arc::new(MultiProgress::new());

    components
        .par_iter()
        .filter(|component| component.component_type == ComponentType::Binary)
        .for_each(|component| {
            build_component(&mpb, component, &config.builder.build_target, &config.builder.rootfs_dir);
        });

    mpb.clear().expect("Failed to clear progress bars");

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_message("Building initrd");
    let initrd = Command::new("sh")
        .args([
            "-c",
            &format!(
                "find . | cpio -o -H newc > {}",
                config.builder.dist_dir.join("iso/boot/initrd").display()
            ),
        ])
        .stderr(Stdio::null())
        .current_dir(&config.builder.rootfs_dir)
        .spawn()
        .expect("Failed to create initrd")
        .wait()
        .expect("Failed to wait for initrd");

    if initrd.success() {
        if iso {
            pb.set_message("Finished building initrd");
        } else {
            pb.finish_with_message("Finished building initrd");
            pb.finish_and_clear();
            return Ok(());
        }
    } else {
        pb.finish_with_message("Failed to build initrd");
        process::exit(1);
    }

    pb.set_message("Building ISO");
    let iso = Command::new("grub-mkrescue")
        .args(["-o", &config.builder.dist_dir.join("PoopyPooOS.iso").display().to_string()])
        .arg(config.builder.dist_dir.join("iso").display().to_string())
        .stderr(Stdio::null())
        .current_dir(&config.builder.dist_dir)
        .spawn()
        .expect("Failed to create ISO")
        .wait()
        .expect("Failed to wait for ISO creation");

    if iso.success() {
        pb.finish_with_message("Finished building ISO");
    } else {
        pb.finish_with_message("Failed to build ISO");
        process::exit(1);
    }

    Ok(())
}

fn build_component(mpb: &Arc<MultiProgress>, component: &Component, target: &str, rootfs_path: &Path) {
    assert!(component.config.is_some(), "Missing config for component: {}", component.name);

    let config = toml::de::from_str::<BinaryComponentConfig>(&component.config.as_ref().unwrap().to_string())
        .expect("Failed to parse component config");

    let target = if let Some(target) = &config.build_target {
        target
    } else {
        &target.to_string()
    };

    compile(mpb, &component.name, &component.path, &config, &target);

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
        return;
    } else {
        change_root(&config.out, rootfs_path)
    };

    let binary_out_directory = binary_out.parent().unwrap();
    fs::create_dir_all(binary_out_directory).expect("Failed to create directory for binary");
    fs::copy(&build_out, &binary_out).expect("Failed to copy binary");

    if !&component.path.join("lib").exists() {
        return;
    }

    let libs = fs::read_dir(&component.path.join("lib"))
        .expect("Failed to read component lib directory")
        .filter_map(Result::ok)
        .filter(|f| f.file_name().to_string_lossy().contains(".so"))
        .collect::<Vec<_>>();

    libs.iter().for_each(|lib| {
        fs::copy(lib.path(), rootfs_path.join("lib").join(lib.file_name())).expect("Failed to copy library from component");
    });
}

fn compile(mpb: &Arc<MultiProgress>, job_name: &str, workspace_path: &Path, config: &BinaryComponentConfig, target: &str) {
    let build_type_args = match config.build_type {
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

    fs::write("log.txt", workspace_path.join("lib").display().to_string()).expect("failed to log sh");

    let mut cargo = Command::new("cargo")
        .args(build_type_args)
        .env(
            "LD_LIBRARY_PATH",
            if workspace_path.join("lib").exists() {
                workspace_path.join("lib").display().to_string()
            } else {
                String::default()
            },
        )
        .args(["--target", target])
        .stderr(Stdio::piped())
        .current_dir(workspace_path)
        .spawn()
        .expect("Failed to wait for cargo build");

    for line in BufReader::new(cargo.stderr.take().unwrap()).lines() {
        let line = line.unwrap().trim().to_string();

        if !line.is_empty() {
            pb.set_message(format!("{job_name}: {line}"));
        }
    }

    mpb.add(pb);

    cargo.wait().unwrap();
}
