# Kaos build script (Phase 1).
# Builds the kernel, assembles the BIOS disk image, and runs host tests.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$manifest = Join-Path $root "Cargo.toml"

# Runs a cargo step, retrying once on transient Windows linker locks
# (LNK1104: antivirus/indexer holding freshly written .exe files).
function Invoke-CargoStep {
    param([string[]]$CargoArgs)
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        $output = & cargo @CargoArgs --manifest-path $manifest --message-format=short 2>&1
        $output | ForEach-Object { "$_" }
        if ($LASTEXITCODE -eq 0) { return }
        $text = ($output | Out-String)
        if ($attempt -lt 3 -and $text -match "LNK1104") {
            Write-Host "Transient linker lock (LNK1104), retrying ($attempt/3)..."
            Start-Sleep -Seconds 3
            continue
        }
        throw "cargo $($CargoArgs -join ' ') failed with exit code $LASTEXITCODE."
    }
}

cargo fmt --check --manifest-path $manifest
Invoke-CargoStep @("build")
Invoke-CargoStep @("test", "-p", "kernel")
Invoke-CargoStep @("clippy", "--workspace", "--all-targets")
Invoke-CargoStep @("clippy", "-p", "kernel", "--target", "x86_64-unknown-none")

Write-Host "Build OK."
