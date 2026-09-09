# Kaos

Hobby operating system in Rust, built from scratch (`#![no_std]`) for `x86_64`.
Goal: boot in QEMU with its own GUI (terminal, file explorer, minimal browser).

Status: Phase 1 — kernel boots in QEMU (BIOS) and prints "Hello, Kaos!" on
the framebuffer plus serial logs.

## Prerequisites

- Rust nightly (pinned automatically via `rust-toolchain.toml`)
- `llvm-tools-preview` (listed in `rust-toolchain.toml`, installed via rustup)
- QEMU: `winget install --id SoftwareFreedomConservancy.QEMU -e --silent`
  (ensure `qemu-system-x86_64` is on `PATH`)

## Quickstart

```powershell
./scripts/build.ps1            # fmt + build images + tests + clippy
./scripts/run.ps1              # boot the BIOS image in QEMU
./scripts/run.ps1 -- -display none  # headless boot (serial logs only)
./scripts/run.ps1 -Iso .\some-third-party.iso  # third-party ISO smoke test
```

## Layout

```
kernel/src/main.rs   # entry point: serial + framebuffer init, then halt
kernel/src/serial.rs # COM1 logging (uart_16550)
kernel/src/framebuffer.rs  # framebuffer text writer (unit tested on host)
src/main.rs          # host launcher: boots images in QEMU
build.rs             # assembles BIOS/UEFI disk images from the kernel ELF
scripts/             # build.ps1, run.ps1 (Windows PowerShell)
```
