# Installe en silence, vérifie les fichiers, le raccourci et le démarrage du démon, puis désinstalle et vérifie le nettoyage.
param([Parameter(Mandatory = $true)][string]$Setup)
$ErrorActionPreference = 'Stop'
$tmp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$dir = Join-Path $tmp 'mira-install-test'
$data = Join-Path $tmp 'mira-data-test'
function Wait-Until([scriptblock]$Condition, [int]$Seconds, [string]$What) {
    $end = (Get-Date).AddSeconds($Seconds)
    while ((Get-Date) -lt $end) { if (& $Condition) { return }; Start-Sleep -Milliseconds 500 }
    throw "Délai dépassé : $What"
}
Start-Process $Setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/DIR=`"$dir`"" -Wait
foreach ($name in 'mira.exe', 'mira-daemon.exe', 'mira_bridge.dll', 'vcruntime140.dll', 'msvcp140.dll') {
    if (-not (Test-Path (Join-Path $dir $name))) { throw "Fichier installé absent : $name" }
}
$shortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\MIRA.lnk'
if (-not (Test-Path $shortcut)) { throw 'Raccourci du menu Démarrer absent.' }
$daemon = Start-Process (Join-Path $dir 'mira-daemon.exe') -ArgumentList '--home', "`"$data`"", '--exit-when-idle' -PassThru
$portFile = Join-Path $data 'profiles\default\daemon.port'
Wait-Until { Test-Path $portFile } 60 'le démon publie son port'
$port = [int](Get-Content $portFile)
$client = New-Object System.Net.Sockets.TcpClient
$client.Connect('127.0.0.1', $port)
$client.Close()
Write-Host "Démon joignable sur 127.0.0.1:$port"
Stop-Process -Id $daemon.Id -Force
& (Join-Path $dir 'unins000.exe') /VERYSILENT /SUPPRESSMSGBOXES | Out-Null
Wait-Until { -not (Test-Path (Join-Path $dir 'mira.exe')) } 60 'la désinstallation retire mira.exe'
Wait-Until { -not (Test-Path $shortcut) } 30 'la désinstallation retire le raccourci'
Write-Host 'Installeur : installation, démon et désinstallation vérifiés.'
