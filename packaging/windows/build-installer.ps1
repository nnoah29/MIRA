# Prépare le dossier à installer (app + pont + démon + runtime Visual C++), puis produit le zip portable et l'installeur.
# Prérequis : packaging\windows\build.ps1 déjà exécuté, et Inno Setup 6 (ISCC.exe) installé.
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$OutDir = (Join-Path $PSScriptRoot '..\out')
)
$ErrorActionPreference = 'Stop'
$root = Resolve-Path "$PSScriptRoot\..\.."
$release = Join-Path $root 'app\build\windows\x64\runner\Release'
foreach ($name in 'mira.exe', 'mira_bridge.dll', 'mira-daemon.exe') {
    if (-not (Test-Path (Join-Path $release $name))) { throw "$name absent de $release : lancer build.ps1 d'abord." }
}
$stage = Join-Path $root 'app\build\windows\stage'
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
Copy-Item $release $stage -Recurse
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products * -property installationPath
$crt = Get-ChildItem (Join-Path $vs 'VC\Redist\MSVC') -Recurse -Directory -Filter 'Microsoft.VC*.CRT' |
    Where-Object { $_.FullName -match '\\x64\\' -and $_.FullName -notmatch 'onecore|debug_nonredist' } |
    Sort-Object FullName | Select-Object -Last 1
if (-not $crt) { throw 'Runtime Visual C++ (x64) introuvable.' }
Copy-Item (Join-Path $crt.FullName '*.dll') $stage
New-Item -ItemType Directory -Force $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path
$zip = Join-Path $OutDir "mira-mail-$Version-windows-x64.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path "$stage\*" -DestinationPath $zip
$iscc = (Get-Command ISCC.exe -ErrorAction SilentlyContinue).Source
if (-not $iscc) { $iscc = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe' }
if (-not (Test-Path $iscc)) { throw 'Inno Setup 6 (ISCC.exe) introuvable.' }
& $iscc "/DVersion=$Version" "/DStage=$stage" "/DOut=$OutDir" "$PSScriptRoot\mira.iss"
if ($LASTEXITCODE -ne 0) { throw "ISCC a échoué ($LASTEXITCODE)." }
Write-Host "Terminé : $zip et $OutDir\mira-mail-$Version-windows-x64-setup.exe"
