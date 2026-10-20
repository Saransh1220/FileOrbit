<#
.SYNOPSIS
  Installs Radial Convert on Windows: the app, the `radial` command, and
  (optionally) the converters you pick.

.DESCRIPTION
  irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex

  Options cannot be passed through `| iex`, so use environment variables for
  that form (set them first in the same window):
    $env:RADIAL_NO_APP = "1"           install only the `radial` command
    $env:RADIAL_NO_PACKS = "1"         do not offer the converter picker
    $env:RADIAL_PACKS = "audio,images" install these packs without asking

  Or download the file and run it with switches:
    .\install.ps1 -NoApp -Packs audio,images

  Everything downloaded is checked against the release's SHA256SUMS before use.
  Other overrides: RADIAL_BASE_URL, RADIAL_INSTALL_DIR.

  Tested on Windows 11 against a fake release (scripts/test_install_ps1.py): a
  normal install, a tampered CLI, a tampered app, a file missing from the
  checksums, the placeholder address, and an unreachable server. Not yet run
  against a real GitHub release.
#>
[CmdletBinding()]
param(
  [switch]$NoApp,
  [switch]$NoPacks,
  [string[]]$Packs
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'   # Invoke-WebRequest is far faster without the progress bar

# The release these files come from. Set this once the repo exists; the script
# refuses to run while it still contains the placeholder.
$BaseUrl = if ($env:RADIAL_BASE_URL) { $env:RADIAL_BASE_URL } else { 'https://github.com/OWNER/REPO/releases/latest/download' }
$CliDir  = if ($env:RADIAL_INSTALL_DIR) { $env:RADIAL_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Radial Convert\bin' }
$AppAsset = 'Radial-Convert-windows-x86_64-setup.exe'
$CliAsset = 'radial-windows-x86_64.exe'

if ($env:RADIAL_NO_APP -eq '1') { $NoApp = $true }
if ($env:RADIAL_NO_PACKS -eq '1') { $NoPacks = $true }
if (-not $Packs -and $env:RADIAL_PACKS) { $Packs = $env:RADIAL_PACKS -split ',' }

function Fail([string]$message) { throw "error: $message" }

if ($BaseUrl -like '*OWNER/REPO*') {
  Fail 'this script has not been pointed at a release yet. Set RADIAL_BASE_URL to the release download address.'
}

switch ($env:PROCESSOR_ARCHITECTURE) {
  'AMD64' { }
  default { Fail "unsupported processor: $($env:PROCESSOR_ARCHITECTURE). Only 64-bit Intel/AMD Windows is supported for now." }
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("radial-install-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
  function Fetch([string]$name) {
    $dest = Join-Path $tmp $name
    try { Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$name" -OutFile $dest }
    catch { Fail "could not download $name from $BaseUrl ($($_.Exception.Message))" }
    return $dest
  }

  # Throws unless the file's SHA-256 equals the entry for the name in SHA256SUMS.
  function Verify([string]$path, [string]$name) {
    $line = Get-Content (Join-Path $tmp 'SHA256SUMS') | Where-Object { $_ -match "^\S+\s+\*?$([regex]::Escape($name))$" } | Select-Object -First 1
    if (-not $line) { Fail "$name is not listed in SHA256SUMS" }
    $want = ($line -split '\s+')[0]
    $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash
    if ($want -ne $got) { Fail "checksum mismatch for $name (expected $want, got $got). Nothing was installed." }
  }

  Write-Host 'Downloading checksums'
  [void](Fetch 'SHA256SUMS')

  Write-Host "Downloading $CliAsset"
  $cli = Fetch $CliAsset
  Verify $cli $CliAsset

  $app = $null
  if (-not $NoApp) {
    Write-Host "Downloading $AppAsset"
    $app = Fetch $AppAsset
    Verify $app $AppAsset
  }

  # Nothing is installed until every download has been verified.
  New-Item -ItemType Directory -Force -Path $CliDir | Out-Null
  Copy-Item -Force $cli (Join-Path $CliDir 'radial.exe')
  Write-Host "Installed radial to $CliDir\radial.exe"

  # Put the command on the user's PATH (per user, no administrator needed).
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (($userPath -split ';') -notcontains $CliDir) {
    [Environment]::SetEnvironmentVariable('Path', ($userPath.TrimEnd(';') + ';' + $CliDir), 'User')
    Write-Host 'Added it to your PATH (open a new terminal to use it there).'
  }
  $env:Path = "$env:Path;$CliDir"

  if ($app) {
    Write-Host 'Installing Radial Convert (silent, for this user only)'
    $proc = Start-Process -FilePath $app -ArgumentList '/S' -Wait -PassThru
    if ($proc.ExitCode -ne 0) { Fail "the installer exited with code $($proc.ExitCode)" }
  }

  Write-Host ''
  if ($Packs) {
    & (Join-Path $CliDir 'radial.exe') install --packs ($Packs -join ',') --yes
  }
  elseif (-not $NoPacks) {
    & (Join-Path $CliDir 'radial.exe') install
  }

  Write-Host ''
  Write-Host 'Done. Open Radial Convert, then hold Shift and drag a file.'
}
finally {
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
}
