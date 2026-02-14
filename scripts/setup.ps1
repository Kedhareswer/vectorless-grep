$ErrorActionPreference = "Stop"

function Add-PathIfMissing {
  param([string]$PathSegment)
  if (-not $PathSegment) {
    return
  }
  if (-not ($env:PATH -split ";" | Where-Object { $_ -eq $PathSegment })) {
    $env:PATH = "$PathSegment;$env:PATH"
  }
}

function Refresh-PathFromEnvironment {
  $machinePath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
  $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
  $env:PATH = "$machinePath;$userPath"
}

function Ensure-PackageManager {
  $hasWinget = [bool](Get-Command winget -ErrorAction SilentlyContinue)
  $hasChoco = [bool](Get-Command choco -ErrorAction SilentlyContinue)

  if (-not $hasWinget -and -not $hasChoco) {
    Write-Host "Neither winget nor choco detected. Falling back to direct installers." -ForegroundColor Yellow
  }

  return [PSCustomObject]@{
    Winget = $hasWinget
    Choco  = $hasChoco
  }
}

function Test-IsAdministrator {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = [Security.Principal.WindowsPrincipal]::new($identity)
  return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Resolve-ToolSource {
  param(
    [string]$CommandName,
    [string]$DisplayName
  )

  $cmd = Get-Command $CommandName -ErrorAction SilentlyContinue
  if ($cmd) {
    Write-Host "$DisplayName already installed." -ForegroundColor DarkGray
    return $true
  }
  return $false
}

function Install-WithWinget {
  param([string]$WingetId)

  try {
    winget install --id $WingetId --exact --silent --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE -ne 0) {
      throw "winget exit code $LASTEXITCODE"
    }
    Refresh-PathFromEnvironment
    return $true
  }
  catch {
    Write-Host "winget install failed for '$WingetId': $($_.Exception.Message)" -ForegroundColor Yellow
    return $false
  }
}

function Install-WithChoco {
  param([string]$ChocoPackage)

  try {
    choco install $ChocoPackage -y --no-progress
    if ($LASTEXITCODE -ne 0) {
      throw "choco exit code $LASTEXITCODE"
    }
    Refresh-PathFromEnvironment
    return $true
  }
  catch {
    Write-Host "choco install failed for '$ChocoPackage': $($_.Exception.Message)" -ForegroundColor Yellow
    return $false
  }
}

function Invoke-DirectInstaller {
  param(
    [string]$DownloadUrl,
    [string]$DisplayName,
    [string[]]$InstallerArgs,
    [ValidateSet("exe", "msi")]
    [string]$InstallerType
  )

  if (-not $DownloadUrl) {
    return $false
  }

  $extension = if ($InstallerType -eq "msi") { ".msi" } else { ".exe" }
  $tempFile = Join-Path $env:TEMP ("vectorless-" + [guid]::NewGuid().ToString() + $extension)

  try {
    Write-Host "Downloading $DisplayName installer..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $tempFile -UseBasicParsing

    Write-Host "Running $DisplayName installer..." -ForegroundColor Cyan
    if ($InstallerType -eq "msi") {
      $args = @("/i", $tempFile, "/qn", "/norestart")
      $proc = Start-Process -FilePath "msiexec.exe" -ArgumentList $args -Wait -PassThru
    }
    else {
      $proc = Start-Process -FilePath $tempFile -ArgumentList $InstallerArgs -Wait -PassThru
    }

    if ($proc.ExitCode -ne 0 -and $proc.ExitCode -ne 3010) {
      throw "$DisplayName installer exit code $($proc.ExitCode)"
    }

    Refresh-PathFromEnvironment
    return $true
  }
  catch {
    Write-Host "Direct install failed for '$DisplayName': $($_.Exception.Message)" -ForegroundColor Yellow
    return $false
  }
  finally {
    if (Test-Path $tempFile) {
      Remove-Item $tempFile -Force
    }
  }
}

function Get-NodeLtsMsiUrl {
  try {
    $index = Invoke-RestMethod -Uri "https://nodejs.org/dist/index.json"
    $latestLts = $index | Where-Object { $_.lts -and ($_.files -contains "win-x64-msi") } | Select-Object -First 1
    if (-not $latestLts) {
      return $null
    }
    return "https://nodejs.org/dist/$($latestLts.version)/node-$($latestLts.version)-x64.msi"
  }
  catch {
    return $null
  }
}

function Ensure-ToolAuto {
  param(
    [string]$CommandName,
    [string]$DisplayName,
    [string]$WingetId,
    [string]$ChocoPackage,
    [string]$DirectUrl,
    [string[]]$DirectInstallerArgs,
    [ValidateSet("exe", "msi")]
    [string]$DirectInstallerType = "exe",
    [string]$PostInstallPath
  )

  if (Resolve-ToolSource -CommandName $CommandName -DisplayName $DisplayName) {
    return
  }

  Write-Host "$DisplayName is missing. Attempting automatic install..." -ForegroundColor Cyan
  $installed = $false

  if (-not $installed -and $script:PackageManagers.Winget -and $WingetId) {
    Write-Host "Trying winget for $DisplayName..." -ForegroundColor DarkGray
    $installed = Install-WithWinget -WingetId $WingetId
  }

  if (-not $installed -and $script:PackageManagers.Choco -and $ChocoPackage) {
    Write-Host "Trying choco for $DisplayName..." -ForegroundColor DarkGray
    $installed = Install-WithChoco -ChocoPackage $ChocoPackage
  }

  if (-not $installed -and $DirectUrl) {
    Write-Host "Trying direct installer for $DisplayName..." -ForegroundColor DarkGray
    $installed = Invoke-DirectInstaller -DownloadUrl $DirectUrl -DisplayName $DisplayName -InstallerArgs $DirectInstallerArgs -InstallerType $DirectInstallerType
  }

  if ($PostInstallPath) {
    Add-PathIfMissing $PostInstallPath
  }
  Refresh-PathFromEnvironment

  if (-not (Get-Command $CommandName -ErrorAction SilentlyContinue)) {
    throw "Unable to install $DisplayName automatically. Please install it manually, then rerun setup."
  }

  Write-Host "$DisplayName installation verified." -ForegroundColor Green
}

function Test-MSVCBuildToolsInstalled {
  $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
  if (-not (Test-Path $vswhere)) {
    return $false
  }

  $installPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
  return ($LASTEXITCODE -eq 0 -and [bool]$installPath)
}

function Ensure-MSVCBuildToolsAuto {
  if (Test-MSVCBuildToolsInstalled) {
    Write-Host "MSVC Build Tools already installed." -ForegroundColor DarkGray
    return
  }

  Write-Host "MSVC Build Tools missing. Attempting automatic install..." -ForegroundColor Cyan
  $installed = $false

  if ($script:PackageManagers.Winget) {
    Write-Host "Trying winget for Visual Studio Build Tools..." -ForegroundColor DarkGray
    $installed = Install-WithWinget -WingetId "Microsoft.VisualStudio.2022.BuildTools"
  }

  if (-not $installed -and $script:PackageManagers.Choco) {
    Write-Host "Trying choco for Visual Studio Build Tools..." -ForegroundColor DarkGray
    $installed = Install-WithChoco -ChocoPackage "visualstudio2022buildtools"
  }

  if (-not $installed) {
    Write-Host "Trying direct installer for Visual Studio Build Tools..." -ForegroundColor DarkGray
    $installed = Invoke-DirectInstaller `
      -DownloadUrl "https://aka.ms/vs/17/release/vs_BuildTools.exe" `
      -DisplayName "Visual Studio Build Tools" `
      -InstallerType "exe" `
      -InstallerArgs @(
        "--quiet",
        "--wait",
        "--norestart",
        "--nocache",
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
      )
  }

  if (-not (Test-MSVCBuildToolsInstalled)) {
    throw "Visual Studio Build Tools installation could not be verified. Re-run in an elevated terminal or install Build Tools manually."
  }

  Write-Host "MSVC Build Tools installation verified." -ForegroundColor Green
}

Write-Host "Setting up Vectorless on Windows..." -ForegroundColor Cyan
if (-not (Test-IsAdministrator)) {
  Write-Host "Non-admin shell detected. Some installers may fail without elevation." -ForegroundColor Yellow
}

$script:PackageManagers = Ensure-PackageManager

$nodeMsiUrl = Get-NodeLtsMsiUrl
Ensure-ToolAuto `
  -CommandName "node" `
  -DisplayName "Node.js LTS" `
  -WingetId "OpenJS.NodeJS.LTS" `
  -ChocoPackage "nodejs-lts" `
  -DirectUrl $nodeMsiUrl `
  -DirectInstallerType "msi" `
  -DirectInstallerArgs @()

Ensure-ToolAuto `
  -CommandName "npm" `
  -DisplayName "npm" `
  -WingetId "OpenJS.NodeJS.LTS" `
  -ChocoPackage "nodejs-lts" `
  -DirectUrl $nodeMsiUrl `
  -DirectInstallerType "msi" `
  -DirectInstallerArgs @()

Ensure-ToolAuto `
  -CommandName "rustup" `
  -DisplayName "Rustup" `
  -WingetId "Rustlang.Rustup" `
  -ChocoPackage "rustup.install" `
  -DirectUrl "https://win.rustup.rs/x86_64" `
  -DirectInstallerType "exe" `
  -DirectInstallerArgs @("-y") `
  -PostInstallPath "$env:USERPROFILE\.cargo\bin"

Ensure-ToolAuto `
  -CommandName "cargo" `
  -DisplayName "Cargo" `
  -WingetId "Rustlang.Rustup" `
  -ChocoPackage "rustup.install" `
  -DirectUrl "https://win.rustup.rs/x86_64" `
  -DirectInstallerType "exe" `
  -DirectInstallerArgs @("-y") `
  -PostInstallPath "$env:USERPROFILE\.cargo\bin"

Ensure-MSVCBuildToolsAuto

Write-Host "Installing Node dependencies..." -ForegroundColor Cyan
if (Test-Path "package-lock.json") {
  npm ci
}
else {
  npm install
}

Write-Host "Installing Rust formatting component..." -ForegroundColor Cyan
rustup component add rustfmt

Write-Host "Setup complete." -ForegroundColor Green
Write-Host "Run: npm run tauri:dev" -ForegroundColor Green
