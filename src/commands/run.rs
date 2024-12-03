use crate::config::Config;
use log::{error, info, trace};
use std::process::Stdio;
use tokio::process::Command;

const QEMU_DEFAULT_ARGS: &[&str] = &["-enable-kvm", "-m", "512M", "-smp", "2"];

pub async fn run(config: &Config, use_iso: bool) {
    info!("Starting QEMU");

    // Get QEMU binary
    let qemu_bin = if let Some(bin) = &config.runner.qemu_bin {
        bin.to_string_lossy().to_string()
    } else {
        "qemu-system-x86_64".to_string()
    };

    // Compose QEMU command
    let mut qemu = Command::new(qemu_bin);
    let qemu_args = if let Some(args) = &config.runner.qemu_args {
        &args.iter().map(String::as_str).collect::<Vec<_>>()
    } else {
        QEMU_DEFAULT_ARGS
    };

    qemu.args(qemu_args);
    qemu.stdout(Stdio::inherit());
    qemu.stderr(Stdio::inherit());
    qemu.stdin(Stdio::inherit());

    let dist = &config.builder.dist_dir;

    if use_iso {
        trace!("Using ISO image");
        qemu.args(["-cdrom", &dist.join("PoopyPooOS.iso").display().to_string()]);
    } else {
        qemu.args(["-kernel", &dist.join("iso/boot/kernel").display().to_string()]);
        qemu.args(["-initrd", &dist.join("iso/boot/initrd").display().to_string()]);
        qemu.args(["-append", &config.runner.kernel_args]);
    }

    // Spawn QEMU
    let mut child = match qemu.spawn() {
        Ok(child) => child,
        Err(err) => {
            error!("Failed to start QEMU: {err}");
            return;
        }
    };

    match child.wait().await {
        Ok(status) => {
            if !status.success() {
                error!("QEMU exited with status {status}");
            }
        }
        Err(err) => error!("Failed to wait for QEMU: {err}"),
    }
}
