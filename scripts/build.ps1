# Kaos build script (Phase 0).
# Type-checks and builds the freestanding stub. Booting in QEMU arrives in Phase 1.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

cargo fmt --check
cargo check --message-format=short
cargo build --message-format=short
cargo clippy --message-format=short

Write-Host "Build OK."
