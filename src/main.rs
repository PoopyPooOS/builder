#![feature(ascii_char)]

use serde::Deserialize;
use std::{
    fs,
    process::{Command, Stdio},
    thread,
};

mod builder;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    build_type: builder::BuildType,
    components_dir: String,
    rootfs_dir: String,
    dist_dir: String,
    qemu_args: Vec<String>,
}

const DISTRO_NAME: &str = "PoopyPooOS";
const BUILDER_CONFIG_NAME: &str = "../builder.toml";

fn main() {
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
                builder::build(component.file_name().to_str().unwrap(), config);
            })
        })
        .collect();

    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(err) => {
                if let Some(panic_message) = err.downcast_ref::<&str>() {
                    println!("Build thread panicked with message: {}", panic_message);
                } else {
                    println!("Build thread panicked with an unknown message.");
                }
            }
        }
    }

    println!("\nAll components built successfully!\n");
    println!("Building initrd...");

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_change_root() {
        assert_eq!(
            builder::change_root(
                PathBuf::from("/init"),
                PathBuf::from("/home/real/projects/distro2/rootfs"),
            ),
            PathBuf::from("/home/real/projects/distro2/rootfs/init")
        );
    }
}
