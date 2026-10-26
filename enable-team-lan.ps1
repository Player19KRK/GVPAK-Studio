param([string]$NetworkAlias='', [string]$TrustedNetworkName='')
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (!$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
 Start-Process powershell.exe -Verb RunAs -WindowStyle Normal -ArgumentList ('-NoProfile -ExecutionPolicy Bypass -File "' + $PSCommandPath + '"' + $(if ($NetworkAlias) { ' -NetworkAlias "' + $NetworkAlias + '" -TrustedNetworkName "' + $TrustedNetworkName + '"' } else { '' }))
 exit
}
if ($NetworkAlias) {
 $network = Get-NetConnectionProfile -InterfaceAlias $NetworkAlias -ErrorAction Stop
 if ($TrustedNetworkName -and $network.Name -ne $TrustedNetworkName) { throw 'The Wi-Fi network has changed. Review its trust level in Windows Settings before enabling access.' }
 $network | Set-NetConnectionProfile -NetworkCategory Private
}
$exe = Join-Path $PSScriptRoot 'bin/gvpak-studio.exe'
$rule = Get-NetFirewallRule -Name 'GVPAK-Studio-Team-HTTPS' -ErrorAction SilentlyContinue
if ($rule) { $rule | Get-NetFirewallApplicationFilter | Set-NetFirewallApplicationFilter -Program $exe | Out-Null }
if (!$rule) { New-NetFirewallRule -Name 'GVPAK-Studio-Team-HTTPS' -DisplayName 'GVPAK Studio Team HTTPS (Private LAN only)' -Direction Inbound -Action Allow -Protocol TCP -LocalPort 8788 -RemoteAddress LocalSubnet -Profile Private -Program $exe | Out-Null }
Write-Host 'Private-LAN access enabled on HTTPS port 8788. Public networks remain blocked.'
Read-Host 'Press Enter to close'
