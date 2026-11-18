$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)

$cargoPath = Join-Path $root 'Cargo.toml'
$rootCargo = [System.IO.File]::ReadAllText($cargoPath, $utf8)
$m = [regex]::Match($rootCargo, '(?m)^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"')
if (-not $m.Success) {
    throw "Could not find version in $cargoPath"
}

$maj = [int]$m.Groups[1].Value
$min = [int]$m.Groups[2].Value
$pat = [int]$m.Groups[3].Value
$oldVer = "$maj.$min.$pat"

$pat++
if ($pat -ge 10) {
    $pat = 0
    $min++
}
if ($min -ge 10) {
    $min = 0
    $maj++
}
$newVer = "$maj.$min.$pat"

function Update-Ver {
    param(
        [string]$RelPath,
        [string]$Pattern,
        [string]$Replacement
    )
    $fullPath = Join-Path $root $RelPath
    if (Test-Path $fullPath) {
        $content = [System.IO.File]::ReadAllText($fullPath, $utf8)
        $updated = [regex]::Replace($content, $Pattern, $Replacement, [System.Text.RegularExpressions.RegexOptions]::Multiline)
        [System.IO.File]::WriteAllText($fullPath, $updated, $utf8)
    }
}

Update-Ver 'Cargo.toml' '^version\s*=\s*"\d+\.\d+\.\d+"' "version = `"$newVer`""
Update-Ver 'apps\sekonic-bridge\package.json' '"version":\s*"\d+\.\d+\.\d+"' "`"version`": `"$newVer`""
Update-Ver 'apps\sekonic-bridge\src-tauri\Cargo.toml' '^version\s*=\s*"\d+\.\d+\.\d+"' "version = `"$newVer`""
Update-Ver 'apps\sekonic-bridge\src-tauri\tauri.conf.json' '"version":\s*"\d+\.\d+\.\d+"' "`"version`": `"$newVer`""
Update-Ver 'apps\sekonic-bridge\msix\AppxManifest.xml' 'Version="\d+\.\d+\.\d+\.\d+"' "Version=`"$newVer.0`""
Update-Ver 'packages\sekonic-web-sdk\Cargo.toml' '^version\s*=\s*"\d+\.\d+\.\d+"' "version = `"$newVer`""
Update-Ver 'packages\sekonic-web-sdk\package.json' '"version":\s*"\d+\.\d+\.\d+"' "`"version`": `"$newVer`""
Update-Ver 'apps\sekonic-bridge\src\components\layout\SidebarNav.tsx' 'v\d+\.\d+\.\d+' "v$newVer"

Write-Output "$oldVer to $newVer"
