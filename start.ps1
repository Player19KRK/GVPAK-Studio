param([switch]$NoBrowser)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$url = 'http://127.0.0.1:8787/'
$lanAddresses = @([System.Net.Dns]::GetHostAddresses([System.Net.Dns]::GetHostName()) | Where-Object { $_.AddressFamily -eq [System.Net.Sockets.AddressFamily]::InterNetwork -and ![System.Net.IPAddress]::IsLoopback($_) } | ForEach-Object { $_.IPAddressToString })
$env:GVPAK_TEAM_HOSTS = (@('localhost','127.0.0.1') + $lanAddresses | Select-Object -Unique) -join ','

$exe = Join-Path $PSScriptRoot 'bin/gvpak-studio.exe'
function Test-StudioReady {
 try {
  $health = Invoke-RestMethod ($url + 'api/health') -TimeoutSec 1
  return ($health.service -eq 'gvpak-studio' -and $health.version -eq '2.0.0')
 } catch { return $false }
}
if (!(Test-StudioReady)) {
 if (!(Test-Path -LiteralPath $exe)) { throw 'Missing bin/gvpak-studio.exe. Extract the complete application folder first.' }
 if (!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'dist/index.html'))) { throw 'Missing dist/index.html. Extract the complete application folder first.' }
 if (Get-NetTCPConnection -LocalPort 8787 -State Listen -ErrorAction SilentlyContinue) { throw 'Port 8787 is occupied by another service. Close that service and try again.' }
 Write-Host 'Starting GVPAK Studio...'
 $server = Start-Process -FilePath $exe -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -PassThru
 $ready = $false
 for ($i = 0; $i -lt 30; $i++) {
  if (Test-StudioReady) { $ready = $true; break }
  if ($server.HasExited) { throw "GVPAK Studio stopped during startup (exit $($server.ExitCode))." }
  Start-Sleep -Milliseconds 300
 }
 if (!$ready) { throw 'GVPAK Studio did not become ready at http://127.0.0.1:8787/.' }
}
if (!$NoBrowser) { Start-Process $url }
Write-Host ('GVPAK Studio is ready: ' + $url)

foreach ($address in $lanAddresses) { Write-Host ('Team: https://' + $address + ':8788') }
