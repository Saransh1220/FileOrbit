<#
  Keeps Rust build folders from growing without limit.

  Cargo never deletes the build files of older versions of your code, so a `target` folder
  only grows. This removes the incremental cache and any build file nobody has used for a few
  days, when the folder is over a size limit. Safe: the next build recreates what it needs.

  Usage (from the repository root):
    powershell -File scripts\trim_target.ps1             # trims when a folder is over 4 GB
    powershell -File scripts\trim_target.ps1 -LimitGB 2  # a stricter limit
    powershell -File scripts\trim_target.ps1 -Clean      # delete both target folders outright
  Close the running app first (a running app.exe cannot be deleted).
#>
param(
    [double]$LimitGB = 4,
    [int]$KeepDays = 3,
    [switch]$Clean
)

$root = Split-Path -Parent $PSScriptRoot
$targets = @("app\src-tauri\target", "crates\radial-packs\target") | ForEach-Object { Join-Path $root $_ }

function Size-GB($path) {
    if (-not (Test-Path $path)) { return 0 }
    $bytes = (Get-ChildItem $path -Recurse -Force -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
    return [math]::Round(($bytes / 1GB), 2)
}

foreach ($target in $targets) {
    if (-not (Test-Path $target)) { continue }
    $before = Size-GB $target
    if ($Clean) {
        Remove-Item $target -Recurse -Force -ErrorAction SilentlyContinue
        "{0}: {1} GB -> removed" -f $target, $before
        continue
    }
    if ($before -le $LimitGB) {
        "{0}: {1} GB, under the {2} GB limit" -f $target, $before, $LimitGB
        continue
    }
    $cutoff = (Get-Date).AddDays(-$KeepDays)
    foreach ($profile in Get-ChildItem $target -Directory -ErrorAction SilentlyContinue) {
        Remove-Item (Join-Path $profile.FullName "incremental") -Recurse -Force -ErrorAction SilentlyContinue
        foreach ($folder in "deps", "build", ".fingerprint") {
            $path = Join-Path $profile.FullName $folder
            if (Test-Path $path) {
                Get-ChildItem $path -Recurse -Force -File -ErrorAction SilentlyContinue |
                    Where-Object { $_.LastWriteTime -lt $cutoff } |
                    Remove-Item -Force -ErrorAction SilentlyContinue
            }
        }
    }
    "{0}: {1} GB -> {2} GB" -f $target, $before, (Size-GB $target)
}
