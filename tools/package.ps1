<#
.SYNOPSIS
Build pa3ds for release and pack it into one zip a player unpacks and runs.

.DESCRIPTION
Builds the release, checks the executable starts, and writes
target\package\pa3ds-<version>-windows.zip holding:

  pa3ds.exe          the program, with Prism (which reaches the player's screen reader) built in
  README.md          the player's manual
  LICENSE            pa3ds's licence
  licenses\prism\    Prism's licence and those of the libraries built into it: Prism is under the
                     MPL 2.0, which asks that its licence go wherever its code goes

The player's own folder (settings, keys, names, logs) is not in the zip: pa3ds makes it beside
itself on the first run (ADR-0032).

.PARAMETER SkipTests
Do not run the test suite before building.

.EXAMPLE
powershell -ExecutionPolicy Bypass -File tools\package.ps1
#>
param([switch]$SkipTests)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Invoke-Checked([string]$what, [scriptblock]$command) {
    Write-Host "== $what"
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$what failed (exit $LASTEXITCODE)" }
}

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$name = "pa3ds-$version-windows"

if (-not $SkipTests) {
    Invoke-Checked "tests" { cargo test --lib --locked }
}
# Only a packaged pa3ds checks GitHub for a newer release when it starts (src/update.rs).
$env:PA3DS_RELEASE = "1"
Invoke-Checked "release build" { cargo build --release --locked --bin pa3ds }

$release = Join-Path $root "target\release"
if (-not (Test-Path (Join-Path $release "pa3ds.exe"))) { throw "the build left no pa3ds.exe in $release" }
# A start that prints its help reads nothing and says nothing, and proves the executable runs.
Invoke-Checked "the executable starts" { & (Join-Path $release "pa3ds.exe") --help | Out-Null }

$package = Join-Path $root "target\package"
$stage = Join-Path $package $name
$zip = Join-Path $package "$name.zip"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
if (Test-Path $zip) { Remove-Item -Force $zip }
New-Item -ItemType Directory -Force (Join-Path $stage "licenses\prism") | Out-Null

Copy-Item (Join-Path $release "pa3ds.exe") $stage
Copy-Item README.md, LICENSE $stage
$prism = Join-Path $root "vendor\prism"
Copy-Item (Join-Path $prism "LICENSE"), (Join-Path $prism "NOTICE") (Join-Path $stage "licenses\prism")
Copy-Item -Recurse (Join-Path $prism "LICENSES") (Join-Path $stage "licenses\prism\LICENSES")

Write-Host "== zip"
Compress-Archive -Path $stage -DestinationPath $zip -CompressionLevel Optimal
Remove-Item -Recurse -Force $stage

$size = "{0:N1} MB" -f ((Get-Item $zip).Length / 1MB)
Write-Host "wrote $zip ($size)"
