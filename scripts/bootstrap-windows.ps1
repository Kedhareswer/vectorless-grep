$ErrorActionPreference = "Stop"

Write-Host "bootstrap-windows.ps1 now delegates to setup.ps1." -ForegroundColor Yellow
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$setupScript = Join-Path $scriptDir "setup.ps1"

if (-not (Test-Path $setupScript)) {
  throw "Could not find setup script at $setupScript"
}

& $setupScript
