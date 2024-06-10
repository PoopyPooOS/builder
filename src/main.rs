#![feature(ascii_char)]

use clap::Parser;
use colored::{Color, Colorize};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::{
    fs, io,
    process::{self, Command, Stdio},
    sync::Arc,
    thread,
    time::Duration,
};
use termspin::{Group, Line, Loop, SharedFrames};

mod builder;
mod module;
mod spinner;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    build_type: builder::BuildType,
    components_dir: String,
    rootfs_dir: String,
    dist_dir: String,
    qemu_args: Vec<String>,
    kernel_args: String,
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    iso: bool,
}

const BUILDER_CONFIG_NAME: &str = "builder.toml";

fn main() {
    let args = Cli::parse();

    let config = {
        let config_raw = fs::read_to_string(BUILDER_CONFIG_NAME).expect("Failed to read config file");
        let parsed: Config = toml::from_str(&config_raw).expect("Failed to parse config");

        parsed
    };

    let components = fs::read_dir(&config.components_dir)
        .unwrap()
        .map(|f| f.unwrap())
        .filter(|f| f.file_type().unwrap().is_dir())
        .collect::<Vec<_>>();

    let dots = spinner::dots();
    let main_group = Group::new();
    let main_group = SharedFrames::new(main_group);
    let spin_loop = Loop::new(Duration::from_millis(80), main_group.clone());
    let spin_loop_clone = spin_loop.clone();
    thread::spawn(move || spin_loop_clone.run_stream(io::stdout()));

    let compiling_task = Line::new(dots.clone()).with_text("Building components").shared();
    main_group.lock().push(compiling_task.clone());

    let subtask_group = Group::new().with_indent(1).shared();

    let subtasks = (0..components.len())
        .map(|i| {
            let name = components[i].file_name();
            let name = name.to_str().unwrap();

            Line::new(dots.clone()).with_text(&format!("Building {}", name)).shared()
        })
        .collect::<Vec<_>>();

    subtask_group.lock().extend(subtasks.iter().cloned());
    main_group.lock().push(subtask_group.clone());

    let config = Arc::new(config);
    subtasks.par_iter().enumerate().for_each(|(i, subtask)| {
        let component = &components[i];
        let config = Arc::clone(&config);

        let name = component.file_name();
        let name = name.to_str().unwrap();

        builder::build(name, &config);

        subtask
            .lock()
            .set_spinner_visible(false)
            .set_text(&format!("{} Finished building {}", "✓".color(Color::Green), name));
    });

    compiling_task
        .lock()
        .set_spinner_visible(false)
        .set_text(format!("{} Finished building all components", "✓".color(Color::Green)).as_str());

    drop(compiling_task);
    drop(subtask_group);

    println!();

    let initrd_task = Line::new(dots.clone()).with_text("Building initrd").shared();
    main_group.lock().push(initrd_task.clone());
    let initrd = Command::new("sh")
        .args(["-c", &format!("find . | cpio -o -H newc > {}/iso/boot/initrd", config.dist_dir)])
        .current_dir(&config.rootfs_dir)
        .output()
        .expect("Failed to create initrd");

    if initrd.status.success() {
        initrd_task
            .lock()
            .set_spinner_visible(false)
            .set_text(format!("{} Finished building initrd", "✓".color(Color::Green)).as_str());
    } else {
        initrd_task
            .lock()
            .set_spinner_visible(false)
            .set_text(format!("{} There was an error building the initrd.", "✖".color(Color::Red)).as_str());

        process::exit(1);
    }

    drop(initrd_task);

    if args.iso {
        let iso_task = Line::new(dots.clone()).with_text("Building ISO").shared();
        main_group.lock().push(iso_task.clone());

        let grub_mkrescue = Command::new("grub-mkrescue")
            .args(["-o", &format!("{}/PoopyPooOS.iso", config.dist_dir)])
            .arg(&format!("{}/iso", config.dist_dir))
            .current_dir(&config.dist_dir)
            .output()
            .expect("Failed to create ISO with grub-mkrescue");

        if grub_mkrescue.status.success() {
            iso_task
                .lock()
                .set_spinner_visible(false)
                .set_text(format!("{} Finished building ISO", "✓".color(Color::Green)).as_str());
        } else {
            iso_task
                .lock()
                .set_spinner_visible(false)
                .set_text(format!("{} There was an error building the ISO.", "✖".color(Color::Red)).as_str());

            process::exit(1);
        }

        drop(iso_task);
    }

    drop(main_group);
    drop(dots);

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args(&config.qemu_args);
    qemu.stdout(Stdio::inherit());
    qemu.stderr(Stdio::inherit());
    qemu.stdin(Stdio::inherit());

    if args.iso {
        qemu.args(["-cdrom", &format!("{}/PoopyPooOS.iso", config.dist_dir)]);
    } else {
        qemu.args(["-kernel", &format!("{}/iso/boot/kernel", config.dist_dir)]);
        qemu.args(["-initrd", &format!("{}/iso/boot/initrd", config.dist_dir)]);
        qemu.args(["-append", &config.kernel_args]);
    }

    qemu.spawn().unwrap_or_else(|_| panic!("Failed to start qemu")).wait().unwrap();
}
