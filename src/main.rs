#![feature(ascii_char)]

use serde::Deserialize;
use std::{
    env::{self, current_dir, set_current_dir},
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

#[derive(Deserialize)]
struct ComponentConfig {
    out: String,
    binary_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    build_type: BuildType,
    components_dir: String,
    rootfs_dir: String,
    dist_dir: String,
    qemu_args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
enum BuildType {
    Debug,
    Release,
}

const DISTRO_NAME: &str = "PoopyPooOS";
const BUILDER_CONFIG_NAME: &str = "builder.toml";

fn main() {
    let orginal_cwd = current_dir().expect("Failed to get current directory");

    let config = {
        let config_raw =
            fs::read_to_string(BUILDER_CONFIG_NAME).expect("Failed to read config file");
        let parsed: Config = toml::from_str(&config_raw).expect("Failed to parse config");

        parsed
    };

    let components = fs::read_dir(&config.components_dir)
        .unwrap()
        .map(|f| f.unwrap())
        .collect::<Vec<_>>();

    let handles: Vec<_> = components
        .into_iter()
        .map(|component| {
            let config = config.clone();

            thread::spawn(move || {
                build(component.file_name().to_str().unwrap(), config);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nAll components built successfully!\n");
    println!("Building initrd...");
    set_current_dir(orginal_cwd).expect("Failed to set cwd");

    let initrd = Command::new("sh")
        .args([
            "-c",
            &format!("find . | cpio -o -H newc > {}/initrd", config.dist_dir),
        ])
        .current_dir(config.rootfs_dir)
        .output()
        .expect("Failed to create initrd");

    if initrd.status.success() {
        println!("Successfully created initrd!");
    } else {
        panic!("Failed to create initrd");
    }

    println!("Starting {} with qemu-system-x86_64", DISTRO_NAME);
    let mut qemu = Command::new("qemu-system-x86_64")
        .args(["-kernel", &format!("{}/kernel", config.dist_dir)])
        .args(["-initrd", &format!("{}/initrd", config.dist_dir)])
        .args(config.qemu_args)
        .current_dir("..")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to start {} using qemu", DISTRO_NAME));

    qemu.wait().unwrap();
}

fn build(name: &str, config: Config) {
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

    let mut binary_out_directory = PathBuf::from(&build_config.out);
    binary_out_directory.pop();

    println!("Entering directory {}", component_path.display());

    env::set_current_dir(component_path)
        .unwrap_or_else(|_| panic!("Failed to set cwd for component {}", name));

    fs::create_dir_all(binary_out_directory)
        .unwrap_or_else(|_| panic!("Failed to create parent directories for component {}", name));

    println!(
        "Copying {} to {}",
        component_binary_path.display(),
        build_config.out
    );

    fs::copy(component_binary_path, build_config.out)
        .unwrap_or_else(|_| panic!("Failed to copy binary for component {}", name));

    println!("Finished building {}", name);
}
