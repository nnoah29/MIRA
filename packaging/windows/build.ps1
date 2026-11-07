# Construit MIRA pour Windows : pont FFI (avec les identifiants OAuth chiffrés dans le moteur) puis application.
# Les identifiants viennent de MIRA_OAUTH_CONFIG, sinon de %APPDATA%\mira\config.toml (section [oauth.gmail] / [oauth.outlook]).
# Sans fichier, le build réussit mais la connexion Google/Microsoft échouera chez tous les utilisateurs.
$ErrorActionPreference = 'Stop'
$root = Resolve-Path "$PSScriptRoot\..\.."
if (-not $env:MIRA_OAUTH_CONFIG) {
    $default = Join-Path $env:APPDATA 'mira\config.toml'
    if (Test-Path $default) { $env:MIRA_OAUTH_CONFIG = $default }
}
if ($env:MIRA_OAUTH_CONFIG) { Write-Host "Identifiants OAuth intégrés (chiffrés) depuis $env:MIRA_OAUTH_CONFIG" }
else { Write-Warning 'Aucun identifiant OAuth intégré : créer %APPDATA%\mira\config.toml ou définir MIRA_OAUTH_CONFIG.' }
# Visual Studio 2026 refuse les en-têtes <experimental/coroutine> du plugin webview_all_windows sans ce drapeau.
if ($env:CL -notmatch '_SILENCE_EXPERIMENTAL_COROUTINE') { $env:CL = "$env:CL /D_SILENCE_EXPERIMENTAL_COROUTINE_DEPRECATION_WARNINGS" }
Push-Location "$root\bridge"; cargo build --release; Pop-Location
Push-Location "$root\app"; flutter pub get; flutter build windows --release; Pop-Location
Write-Host "Terminé : $root\app\build\windows\x64\runner\Release\mira.exe"
