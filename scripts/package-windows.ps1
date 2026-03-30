param(
    [ValidateSet("nsis", "wix")]
    [string]$Format = "nsis"
)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
$WindowsIcon = Join-Path $RootDir "src\assets\icon_1024.ico"

if (-not $IsWindows) {
    throw "This packaging script only supports Windows."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required but was not found in PATH."
}

if (-not (Test-Path $WindowsIcon)) {
    throw "Missing Windows icon: $WindowsIcon"
}

$IconInfo = Get-Item $WindowsIcon
if ($IconInfo.Length -le 0) {
    throw "Windows icon is empty: $WindowsIcon"
}

if (-not (Get-Command cargo-packager -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-packager is not installed. Installing it now..."
    cargo install cargo-packager --locked
}

Push-Location $RootDir
try {
    Write-Host "Packaging MDRS as a Windows installer ($Format)..."
    cargo packager --release --formats $Format
    Write-Host ""
    Write-Host "Done. Check dist/packager/ for the generated Windows installer."
}
finally {
    Pop-Location
}
