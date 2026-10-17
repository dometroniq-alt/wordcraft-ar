<#
.SYNOPSIS
  Build and package WordCraft AR for Windows: a self-contained .msi and a portable .zip.

.DESCRIPTION
  Produces, in $env:DIST (default: dist\release):
    WordCraft-AR-<version>-windows-<arch>.msi            per-machine installer (WiX v5)
    WordCraft-AR-<version>-windows-<arch>-portable.zip   "WordCraft AR.exe" + wordcraft-cli.exe + servicewordar\

  The binaries link the C runtime statically (+crt-static), so neither needs the Visual C++
  redistributable: copy the .msi to a USB stick and it installs on an offline machine.

  Needs: Rust (MSVC toolchain), the Windows SDK (comes with the MSVC Build Tools' recommended
  components) and WiX v5 with its UI and Util extensions:
    dotnet tool install --global wix --version 5.0.2
    wix extension add -g WixToolset.UI.wixext/5.0.2
    wix extension add -g WixToolset.Util.wixext/5.0.2

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File packaging\windows\package-ar.ps1
  powershell -ExecutionPolicy Bypass -File packaging\windows\package-ar.ps1 -SkipBuild
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in one place: [workspace.package] version in the root Cargo.toml.
$Version = $env:WORDCRAFT_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
$MsiVersion = ($Version -split '-')[0]

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'x86' { 'i686-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $env:WORDCRAFT_BUILD_SHA -and (Get-Command git -ErrorAction SilentlyContinue)) {
  $env:WORDCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null)
}
if (-not $env:WORDCRAFT_BUILD_DATE) { $env:WORDCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }

Write-Output "WordCraft AR $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  $env:WORDCRAFT_REQUIRE_WINRES = '1'
  Invoke-Native "cargo build ($Target)" { cargo build --release -p wordcraft -p wordcraft-cli --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"
$Stage = Join-Path $TargetDir "windows-package-ar\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'wordcraft.exe') (Join-Path $Stage 'WordCraft AR.exe')
Copy-Item (Join-Path $Bin 'wordcraft-cli.exe') $Stage

# The engine, without Python caches.
$Engine = Join-Path $TargetDir "windows-package-ar\servicewordar"
Remove-Item -Recurse -Force $Engine -ErrorAction SilentlyContinue
Copy-Item -Recurse (Join-Path $Root 'servicewordar') $Engine
Get-ChildItem $Engine -Recurse -Directory -Filter '__pycache__' | Remove-Item -Recurse -Force
# The wheels are only for machines without the embedded Python (tools\bundle-python.cmd); the
# installer ships the embedded Python instead.
if (Test-Path (Join-Path $Engine 'python\python.exe')) { Remove-Item -Recurse -Force (Join-Path $Engine 'wheels') -ErrorAction SilentlyContinue }

# ---- license RTF for the installer's first page ------------------------------------------------
function ConvertTo-RtfText([string] $Text) {
  $sb = [System.Text.StringBuilder]::new()
  foreach ($ch in $Text.ToCharArray()) {
    $c = [int] $ch
    if ($c -eq 0x5C -or $c -eq 0x7B -or $c -eq 0x7D) { [void] $sb.Append('\').Append($ch) }
    elseif ($c -gt 127) { [void] $sb.Append("\u$(if ($c -gt 32767) { $c - 65536 } else { $c })?") }
    else { [void] $sb.Append($ch) }
  }
  return $sb.ToString()
}
$paragraphs = [System.Collections.Generic.List[string]]::new()
$paragraphs.Add('\b WordCraft AR\b0')
$paragraphs.Add((ConvertTo-RtfText 'WordCraft AR is a fork of WordCraft (https://github.com/storytold/wordcraft) by the ArtCraft Team and the WordCraft contributors, with Arabic typography and service-writing tools added. Like WordCraft it is dual-licensed under the MIT License or the Apache License, Version 2.0, at your option. Both license texts follow. Bundled fonts and other third-party material keep their own open licenses, listed in NOTICE and ATTRIBUTION.md.'))
foreach ($file in 'LICENSE-MIT', 'LICENSE-APACHE') {
  $title = $true
  foreach ($para in ((Get-Content -Raw (Join-Path $Root $file)) -split '\r?\n\s*\r?\n')) {
    $joined = (($para -split '\r?\n') | ForEach-Object { $_.Trim() } | Where-Object { $_ }) -join ' '
    if (-not $joined) { continue }
    $paragraphs.Add($(if ($title) { "\b $(ConvertTo-RtfText $joined)\b0" } else { ConvertTo-RtfText $joined }))
    $title = $false
  }
}
$LicenseRtf = Join-Path $Stage 'license.rtf'
$rtf = '{\rtf1\ansi\ansicpg1252\deff0{\fonttbl{\f0\fswiss\fcharset0 Tahoma;}}\viewkind4\uc1\pard\sa120\f0\fs16 ' +
  ($paragraphs -join "\par`r`n") + "\par`r`n}`r`n"
[System.IO.File]::WriteAllText($LicenseRtf, $rtf, [System.Text.Encoding]::ASCII)

# ---- MSI ---------------------------------------------------------------------------------------
$Msi = Join-Path $Dist "WordCraft-AR-$Version-windows-$Arch.msi"
Invoke-Native 'wix build' {
  wix build (Join-Path $PSScriptRoot 'wordcraft-ar.wxs') -arch $Arch `
    -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -d "Version=$MsiVersion" -d "BinDir=$Stage" -d "EngineDir=$Engine" `
    -d "IconPath=$(Join-Path $Root 'assets\app-icon\wordcraft.ico')" `
    -d "LicenseRtf=$LicenseRtf" `
    -o $Msi
}
Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))

# ---- portable zip ------------------------------------------------------------------------------
$Portable = Join-Path $TargetDir "windows-package-ar\WordCraft-AR-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Portable | Out-Null
Copy-Item (Join-Path $Stage '*.exe') $Portable
Copy-Item -Recurse $Engine (Join-Path $Portable 'servicewordar')
foreach ($f in 'README.md', 'README.ar.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Portable }
}
$Zip = Join-Path $Dist "WordCraft-AR-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Compress-Archive -Path $Portable -DestinationPath $Zip

Invoke-Native 'wordcraft-cli --version' { & (Join-Path $Stage 'wordcraft-cli.exe') --version }
Get-Item $Msi, $Zip | Format-Table Name, Length
