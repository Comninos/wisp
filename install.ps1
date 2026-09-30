# Install wisp: build from a clone (or download the latest release), add a Start Menu shortcut.
#
#   irm https://raw.githubusercontent.com/Comninos/wisp/master/install.ps1 | iex
#   .\install.ps1

$ErrorActionPreference = 'Stop'

$url = 'https://github.com/Comninos/wisp/releases/latest/download/wisp-windows-x86_64.exe'
$dir = Join-Path $env:LOCALAPPDATA 'Programs\wisp'
$exe = Join-Path $dir 'wisp.exe'
$here = $PSScriptRoot

New-Item -ItemType Directory -Force -Path $dir | Out-Null

if ($here -and (Test-Path (Join-Path $here 'Cargo.toml'))) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "need 'cargo' (https://rustup.rs)" }
    Write-Host "building from $here"
    cargo build --release --manifest-path (Join-Path $here 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
    Copy-Item (Join-Path $here 'target\release\wisp.exe') $exe -Force
} else {
    Invoke-WebRequest $url -OutFile $exe -UseBasicParsing
}
Write-Host "installed $exe"

$lnk = Join-Path ([Environment]::GetFolderPath('Programs')) 'wisp.lnk'
$shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk)
$shortcut.TargetPath = $exe
$shortcut.Save()
Write-Host "added Start Menu shortcut $lnk"

$path = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($path -split ';') -notcontains $dir) {
    [Environment]::SetEnvironmentVariable('Path', ("$path;$dir").TrimStart(';'), 'User')
    Write-Host "added $dir to user PATH (open a new terminal)"
}

Write-Host 'run:  wisp'
if ($here -and (Test-Path (Join-Path $here 'Cargo.toml'))) {
    Write-Host "edit $here\src\main.rs, then rerun .\install.ps1"
}
