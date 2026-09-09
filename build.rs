//! Bootloader disk image builder (host build script).
//!
//! Creates the BIOS (MBR) disk image from the kernel artifact and exposes
//! its path to the launcher binary via `KAOS_BIOS_IMAGE`. The image is
//! written to `target/images/` (ignored by git). UEFI image creation is
//! deferred: the UEFI bootloader binary does not link on Windows hosts.

use std::env;
use std::path::PathBuf;

/// Locates the kernel ELF built as an artifact dependency for
/// `x86_64-unknown-none`.
fn kernel_artifact_path() -> PathBuf {
    if let Some(path) = env::var_os("CARGO_BIN_FILE_KERNEL_kernel") {
        return PathBuf::from(path);
    }
    if let Some(path) = env::var_os("CARGO_BIN_FILE_KERNEL") {
        return PathBuf::from(path);
    }
    panic!(
        "kernel artifact not found: build with nightly cargo with `bindeps` enabled (see .cargo/config.toml)"
    );
}

/// Bootloader runtime configuration: keep its own serial and framebuffer
/// logging enabled so the boot handoff is visible on both channels.
fn boot_config() -> bootloader::BootConfig {
    let mut config = bootloader::BootConfig::default();
    config.serial_logging = true;
    config.frame_buffer_logging = true;
    config
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=kernel/");

    let kernel = kernel_artifact_path();
    let image_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("images");
    std::fs::create_dir_all(&image_dir).expect("create image directory");

    let bios_image = image_dir.join("kaos-bios.img");
    bootloader::BiosBoot::new(&kernel)
        .set_boot_config(&boot_config())
        .create_disk_image(&bios_image)
        .expect("create BIOS disk image");

    println!("cargo::rustc-env=KAOS_BIOS_IMAGE={}", bios_image.display());
}
