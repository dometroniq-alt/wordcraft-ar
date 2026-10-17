<#
.SYNOPSIS
  Put an embedded Python (with the ServiceWord-AR libraries) beside the engine, so the installer
  needs nothing on the target machine: no Python, no internet.

.DESCRIPTION
  Downloads the official "embeddable" Python zip from python.org, unpacks it into
  servicewordar\python, enables site-packages, and installs the engine's libraries into it with
  pip (binary wheels only). WordCraft AR looks for servicewordar\python\python.exe first.

  Needs internet and a Python 3 with pip on this machine (to run pip). On an offline machine,
  copy the finished servicewordar\python folder instead.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File packaging\windows\bundle-python.ps1
#>
param(
  [string] $PythonVersion = '3.12.10',
  [string] $Engine = ''
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $Engine) { $Engine = Join-Path $Root 'servicewordar' }
$Target = Join-Path $Engine 'python'
$short = ($PythonVersion -split '\.')[0..1] -join ''
$zip = Join-Path $env:TEMP "python-$PythonVersion-embed-amd64.zip"

Write-Output "==> embedded Python $PythonVersion -> $Target"
if (-not (Test-Path $zip)) {
  Invoke-WebRequest -Uri "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip" -OutFile $zip
}
Remove-Item -Recurse -Force $Target -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Target | Out-Null
Expand-Archive -Force $zip $Target

# The embeddable build ignores site-packages unless its ._pth file says so.
$pth = Join-Path $Target "python$short._pth"
@("python$short.zip", ".", "Lib\site-packages", "import site") | Set-Content -Path $pth -Encoding ASCII

# pip from the host Python, installing INTO the embedded tree (binary wheels only).
$site = Join-Path $Target 'Lib\site-packages'
New-Item -ItemType Directory -Force -Path $site | Out-Null
$host_py = if (Get-Command py -ErrorAction SilentlyContinue) { 'py' } else { 'python' }
$args = @()
if ($host_py -eq 'py') { $args += '-3' }
& $host_py @args -m pip install --quiet --only-binary=:all: --platform win_amd64 --python-version $PythonVersion --implementation cp `
  --target $site lxml uharfbuzz fonttools pypdf reportlab pillow
if ($LASTEXITCODE -ne 0) { throw "pip install failed ($LASTEXITCODE)" }

& (Join-Path $Target 'python.exe') -c "import lxml, uharfbuzz, fontTools, pypdf, reportlab, PIL, sys; print('embedded python ok', sys.version.split()[0])"
if ($LASTEXITCODE -ne 0) { throw 'the embedded Python cannot import the engine libraries' }
$size = (Get-ChildItem $Target -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB
Write-Output ("embedded Python: {0:N1} MB" -f $size)
