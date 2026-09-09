//! Kaos image launcher (host tool).
//!
//! Boots the BIOS disk image assembled by `build.rs` in QEMU.
//! Usage: `cargo run -- bios [-- extra QEMU args...]`.

use std::env;
use std::path::PathBuf;
use std::process::{self, Command};

/// Fallback when QEMU is installed but missing from `PATH`.
const QEMU_FALLBACK: &str = r"C:\Program Files\qemu\qemu-system-x86_64.exe";

/// Resolves the QEMU binary: `PATH` first, well-known install dir as fallback.
fn qemu_binary() -> PathBuf {
    if Command::new("qemu-system-x86_64")
        .arg("--version")
        .output()
        .is_ok()
    {
        PathBuf::from("qemu-system-x86_64")
    } else {
        PathBuf::from(QEMU_FALLBACK)
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "bios".to_string());
    let extra: Vec<String> = args.collect();

    let image = match mode.as_str() {
        "bios" => PathBuf::from(env!("KAOS_BIOS_IMAGE")),
        "uefi" => {
            eprintln!(
                "UEFI boot is deferred: the UEFI bootloader binary does not link on Windows hosts."
            );
            process::exit(2);
        }
        other => {
            eprintln!("Unknown mode {:?}: expected `bios` or `uefi`.", other);
            process::exit(2);
        }
    };

    if !image.exists() {
        eprintln!("Disk image not found at {}.", image.display());
        process::exit(1);
    }

    let mut command = Command::new(qemu_binary());
    command
        .arg("-drive")
        .arg(format!("format=raw,file={}", image.display()))
        .arg("-serial")
        .arg("stdio")
        .arg("-m")
        .arg("512M")
        .arg("-no-reboot")
        .args(&extra);

    let status = command.status().unwrap_or_else(|err| {
        eprintln!("Failed to launch QEMU: {}.", err);
        process::exit(1);
    });
    process::exit(status.code().unwrap_or(1));
}
