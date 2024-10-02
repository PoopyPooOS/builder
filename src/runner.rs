use crate::types::Config;
use std::process::{Command, Stdio};
use which::which;

const QEMU_DEFAULT_ARGS: &[&str] = &["-enable-kvm", "-m", "512M", "-smp", "2"];

pub fn run(config: &Config, iso: bool) {
    let mut qemu = Command::new(config.runner.qemu_bin.as_ref().unwrap_or(&which("qemu-system-x86_64").unwrap()));
    let default_args = QEMU_DEFAULT_ARGS.iter().map(ToString::to_string).collect::<Vec<String>>();
    let qemu_args = &config.runner.qemu_args.as_ref().unwrap_or(&default_args);
    qemu.args(*qemu_args);
    qemu.stdout(Stdio::inherit());
    qemu.stderr(Stdio::inherit());
    qemu.stdin(Stdio::inherit());

    if iso {
        qemu.args(["-cdrom", &config.builder.dist_dir.join("PoopyPooOS.iso").display().to_string()]);
    } else {
        qemu.args(["-kernel", &config.builder.dist_dir.join("iso/boot/kernel").display().to_string()]);
        qemu.args(["-initrd", &config.builder.dist_dir.join("iso/boot/initrd").display().to_string()]);
        qemu.args(["-append", &config.runner.kernel_args]);
    }

    qemu.spawn()
        .expect("Failed to start qemu-system-x86_64")
        .wait()
        .expect("Failed to run qemu-system-x86_64");
}
