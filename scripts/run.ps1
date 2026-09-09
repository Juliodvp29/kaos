# Kaos run script (Phase 1).
# Boots the BIOS image in QEMU via the builder, or a third-party ISO for
# emulation smoke tests. Extra arguments are forwarded to QEMU (BIOS boot)
# or appended after the ISO flags (ISO boot).
# Usage: ./scripts/run.ps1 [-- -display none]
#        ./scripts/run.ps1 -Iso path\to\image.iso
param(
    [string]$Iso = "",
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$QemuArgs = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

if ($Iso -ne "") {
    $qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
    if (-not $qemu) {
        $fallback = "C:\Program Files\qemu\qemu-system-x86_64.exe"
        if (Test-Path -LiteralPath $fallback) {
            $qemu = Get-Command $fallback
        } else {
            throw "qemu-system-x86_64 not found. Install QEMU (winget install SoftwareFreedomConservancy.QEMU) and ensure it is on PATH."
        }
    }
    if (-not (Test-Path -LiteralPath $Iso)) {
        throw "ISO not found: $Iso"
    }
    & $qemu -cdrom $Iso -m 512M -boot d @QemuArgs
} else {
    & cargo run --manifest-path (Join-Path $root "Cargo.toml") -- bios @QemuArgs
}
