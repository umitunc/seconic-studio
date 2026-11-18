$ErrorActionPreference = "Stop"

$BridgeDir = Resolve-Path "$PSScriptRoot\.."
$TauriDir = Join-Path $BridgeDir "src-tauri"
$MsixOut = Join-Path $PSScriptRoot "out"
$Staging = Join-Path $MsixOut "staging"

Write-Host "1/4 Frontend (Vite) derleniyor..."
npm --prefix $BridgeDir run build

Write-Host "2/4 Release EXE (sekonic-bridge.exe) derleniyor..."
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo build --release --manifest-path (Join-Path $TauriDir "Cargo.toml")

$ExePath = Join-Path $TauriDir "target\release\sekonic-bridge.exe"
if (!(Test-Path $ExePath)) {
    throw "sekonic-bridge.exe bulunamadı: $ExePath"
}

Write-Host "3/4 MSIX staging hazırlanıyor..."
Remove-Item -Recurse -Force $Staging -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Staging | Out-Null
Copy-Item $ExePath (Join-Path $Staging "sekonic-bridge.exe")
Copy-Item (Join-Path $PSScriptRoot "AppxManifest.xml") (Join-Path $Staging "AppxManifest.xml")
Copy-Item -Recurse (Join-Path $PSScriptRoot "Assets") (Join-Path $Staging "Assets")

Write-Host "4/4 MakeAppx ile .msix paketi oluşturuluyor..."
$MakeAppx = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter "makeappx.exe" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -like "*\x64\*" } |
    Select-Object -Last 1

$MsixFile = Join-Path $MsixOut "SekonicStudio_0.1.0_x64.msix"
if ($MakeAppx) {
    & $MakeAppx.FullName pack /d $Staging /p $MsixFile /o
    Write-Host "MSIX paketi hazır: $MsixFile"
    Write-Host "Microsoft Store yüklemesi için Partner Center'dan gelen sertifika veya Store otomatik imzalama ile gönderilebilir."
} else {
    Write-Warning "Windows SDK makeappx.exe bulunamadı. Staging dizini hazır: $Staging"
}
