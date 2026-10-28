param([switch]$NoBrowser)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$url = 'http://127.0.0.1:8787/'
$lanAddresses = @([System.Net.Dns]::GetHostAddresses([System.Net.Dns]::GetHostName()) | Where-Object { $_.AddressFamily -eq [System.Net.Sockets.AddressFamily]::InterNetwork -and ![System.Net.IPAddress]::IsLoopback($_) } | ForEach-Object { $_.IPAddressToString })
$env:GVPAK_TEAM_HOSTS = (@('localhost','127.0.0.1') + $lanAddresses | Select-Object -Unique) -join ','
$env:GVPAK_PROFILE = 'dev'
$pythonCandidates = @($env:GVPAK_PYTHON, 'C:\Program Files\Python310\python.exe', (Get-Command python -ErrorAction SilentlyContinue).Source) | Where-Object { $_ } | Select-Object -Unique
$pythonReady = $false
foreach ($pythonCandidate in $pythonCandidates) {
 try { $cvVersion = & $pythonCandidate -I -c 'import cv2;print(cv2.__version__)' 2>$null; if ($LASTEXITCODE -eq 0) { $env:GVPAK_PYTHON = $pythonCandidate; $pythonReady = $true; break } } catch {}
}
if (!$pythonReady) { throw '未找到可用的本地OpenCV环境。请使用已安装cv2的Python设置GVPAK_PYTHON。' }

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
try { $localTools = Invoke-RestMethod ($url + 'api/invoke/assistant_tools') -Method Post -ContentType 'application/json' -Headers @{'X-GVPAK-IPC'='1'} -Body '{}' -TimeoutSec 15; if (!$localTools.available) { throw 'OpenCV unavailable' } } catch { throw '应用已启动，但本地OpenCV检查未通过。请关闭旧版服务后重新一键启动。' }
try { $newProduct = Invoke-RestMethod ($url + 'api/invoke/new_product_capabilities') -Method Post -ContentType 'application/json' -Headers @{'X-GVPAK-IPC'='1'} -Body '{}' -TimeoutSec 5; if (!$newProduct.analysisModel) { throw 'Missing module' } } catch { throw '当前运行的是旧版服务，新品模块未就绪。请关闭旧版服务后重新一键启动。' }
Write-Host ('本地OpenCV已就绪：' + $localTools.version)
if (!$NoBrowser) { Start-Process $url }
Write-Host ('GVPAK Studio is ready: ' + $url)

foreach ($address in $lanAddresses) { Write-Host ('Team: https://' + $address + ':8788') }
