use std::process::{Command, Stdio};

use crate::types::Config;

pub fn run(config: &Config, iso: bool) {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args(&config.qemu_args);
    qemu.stdout(Stdio::inherit());
    qemu.stderr(Stdio::inherit());
    qemu.stdin(Stdio::inherit());

    if iso {
        qemu.args(["-cdrom", &config.dist_dir.join("PoopyPooOS.iso").display().to_string()]);
    } else {
        qemu.args(["-kernel", &config.dist_dir.join("iso/boot/kernel").display().to_string()]);
        qemu.args(["-initrd", &config.dist_dir.join("iso/boot/initrd").display().to_string()]);
        qemu.args(["-append", &config.kernel_args]);
    }

    qemu.spawn()
        .expect("Failed to start qemu-system-x86_64")
        .wait()
        .expect("Failed to run qemu-system-x86_64");
}
