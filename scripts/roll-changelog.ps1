# Roll changelog/unreleased/*.md (except README.md) into CHANGELOG.md Unreleased.
# Maintainer chore: run on a dedicated short branch, not inside a feature PR.
param()

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$UnreleasedDir = Join-Path $Root "changelog\unreleased"
$ArchiveDir = Join-Path $Root "changelog\archive"
$Changelog = Join-Path $Root "CHANGELOG.md"
$Start = "<!-- changelog:unreleased:start -->"
$End = "<!-- changelog:unreleased:end -->"

if (-not (Test-Path $UnreleasedDir)) {
    throw "missing $UnreleasedDir"
}

$fragments = @(Get-ChildItem -Path $UnreleasedDir -Filter "*.md" |
    Where-Object { $_.Name -ne "README.md" } |
    Sort-Object Name -Descending)

if ($fragments.Count -eq 0) {
    Write-Host "No unreleased fragments to roll."
    exit 0
}

$parts = foreach ($f in $fragments) {
    $raw = [System.IO.File]::ReadAllText($f.FullName).Trim()
    if (-not $raw) {
        throw "empty fragment: $($f.Name)"
    }
    $raw
}

$text = [System.IO.File]::ReadAllText($Changelog)
$nl = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
$block = ($parts -join "$nl$nl") + $nl

$startIdx = $text.IndexOf($Start)
$endIdx = $text.IndexOf($End)
if ($startIdx -lt 0 -or $endIdx -lt 0 -or $endIdx -le $startIdx) {
    throw "CHANGELOG.md is missing $Start / $End markers"
}

$insertAt = $startIdx + $Start.Length
# Keep a newline after the start marker, then prepend the new block.
$afterStart = $text.Substring($insertAt, $endIdx - $insertAt)
if ($afterStart.StartsWith("`r`n")) {
    $insertAt += 2
} elseif ($afterStart.StartsWith("`n")) {
    $insertAt += 1
}

$updated = $text.Substring(0, $insertAt) + $block + $nl + $text.Substring($insertAt)
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($Changelog, $updated, $utf8NoBom)

if (-not (Test-Path $ArchiveDir)) {
    New-Item -ItemType Directory -Path $ArchiveDir | Out-Null
}
foreach ($f in $fragments) {
    $dest = Join-Path $ArchiveDir $f.Name
    if (Test-Path $dest) {
        throw "archive already has $($f.Name); rename the fragment or archive copy"
    }
    Move-Item -Path $f.FullName -Destination $dest
}

Write-Host ("Rolled {0} fragment(s) into CHANGELOG.md Unreleased." -f $fragments.Count)
$fragments | ForEach-Object { Write-Host ("  - " + $_.Name) }
