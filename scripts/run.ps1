# Kaos run script (Phase 0).
# Verifies QEMU works. Optionally boots a third-party ISO to prove emulation
# works before our own kernel enters the picture in Phase 1.
# Usage: ./scripts/run.ps1 [-Iso path\to\image.iso]
param(
    [string]$Iso = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    $fallback = "C:\Program Files\qemu\qemu-system-x86_64.exe"
    if (Test-Path -LiteralPath $fallback) {
        $qemu = Get-Command $fallback
    } else {
        throw "qemu-system-x86_64 not found. Install QEMU (winget install SoftwareFreedomConservancy.QEMU) and ensure it is on PATH."
    }
}

& $qemu --version

if ($Iso -ne "") {
    if (-not (Test-Path -LiteralPath $Iso)) {
        throw "ISO not found: $Iso"
    }
    & $qemu -cdrom $Iso -m 512M -boot d
} else {
    Write-Host "QEMU is available. Pass -Iso <path> to boot a third-party image."
}
