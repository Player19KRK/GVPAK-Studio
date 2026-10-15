param([switch]$NoBrowser)
$ErrorActionPreference='Stop'
Set-Location $PSScriptRoot
$exe=Join-Path $PSScriptRoot 'bin/edge-studio.exe'
if(!(Test-Path $exe)){throw 'Missing bin/edge-studio.exe. Build the Rust backend first.'}
if(!(Get-NetTCPConnection -LocalPort 8787 -State Listen -ErrorAction SilentlyContinue)){
 Start-Process -FilePath $exe -WorkingDirectory $PSScriptRoot -WindowStyle Hidden
}
$ready=$false
for($i=0;$i -lt 30;$i++){
 try {$health=Invoke-RestMethod 'http://127.0.0.1:8787/api/health';if($health.service -eq 'edge-rust' -and $health.version -eq '2.0.0'){$ready=$true;break}}catch{}
 Start-Sleep -Milliseconds 200
}
if(!$ready){throw 'EDGE could not start on port 8787.'}
if(!$NoBrowser){Start-Process 'http://127.0.0.1:8787/'}
Write-Output 'GVPAK STUDIO: http://127.0.0.1:8787/'

