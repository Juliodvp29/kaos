# Kaos

Hobby operating system in Rust, built from scratch (`#![no_std]`) for `x86_64`.
Goal: boot in QEMU with its own GUI (terminal, file explorer, minimal browser).
See `docs/ROADMAP.md` for the full plan and `docs/ARCHITECTURE.md` for design.

Status: Phase 0 — environment ready. No bootable kernel yet (that is Phase 1).

## Prerequisites

- Rust nightly (pinned automatically via `rust-toolchain.toml`)
- `llvm-tools-preview` (listed in `rust-toolchain.toml`, installed via rustup)
- QEMU: `winget install --id SoftwareFreedomConservancy.QEMU -e --silent`
  (ensure `qemu-system-x86_64` is on `PATH`)

## Quickstart

```powershell
./scripts/build.ps1            # fmt check + cargo check + build + clippy
./scripts/run.ps1              # verify QEMU is available
./scripts/run.ps1 -Iso .\some-third-party.iso  # prove emulation works
```

## Layout

```
src/main.rs      # freestanding stub (wiring only; real init order in Phase 1+)
scripts/         # build.ps1, run.ps1 (Windows PowerShell)
docs/            # AGENTS.md, ROADMAP.md, ARCHITECTURE.md, LOG.md, skill
```

## Rules for contributors (agents included)

Read `docs/AGENTS.md` first: plan before coding, English in code/comments,
no merged code without tests, no unreviewed `unsafe`, approved stack only.
