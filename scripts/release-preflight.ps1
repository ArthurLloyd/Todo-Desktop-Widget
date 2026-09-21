$ErrorActionPreference = "Stop"

$releaseRoot = Split-Path -Parent $PSScriptRoot
$releaseEnvPath = Join-Path $releaseRoot ".env"

function Import-ReleaseEnvFile {
  param([string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    return
  }

  Get-Content -LiteralPath $Path | ForEach-Object {
    $line = $_.Trim()
    if ($line.Length -eq 0 -or $line.StartsWith("#")) {
      return
    }

    $parts = $line -split "=", 2
    if ($parts.Count -ne 2) {
      return
    }

    $name = $parts[0].Trim()
    $value = $parts[1].Trim().Trim('"').Trim("'")
    if ($name -match "^[A-Za-z_][A-Za-z0-9_]*$") {
      Set-Item -Path "Env:$name" -Value $value
    }
  }
}

function Assert-ReleaseCommand {
  param([string]$Name)

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Required command is unavailable: $Name"
  }
}

Import-ReleaseEnvFile -Path $releaseEnvPath

foreach ($command in @("node", "npm", "rustc", "cargo")) {
  Assert-ReleaseCommand -Name $command
}

if (-not $env:MICROSOFT_CLIENT_ID) {
  throw "MICROSOFT_CLIENT_ID is required for an official release build. Set it in the environment or an ignored .env file."
}
if ($env:MICROSOFT_CLIENT_ID -notmatch "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$") {
  throw "MICROSOFT_CLIENT_ID must be a Microsoft Entra application ID (GUID)."
}
if (-not $env:MICROSOFT_TENANT) {
  $env:MICROSOFT_TENANT = "common"
}

$requiredIcons = @(
  "src-tauri/icons/32x32.png",
  "src-tauri/icons/128x128.png",
  "src-tauri/icons/128x128@2x.png",
  "src-tauri/icons/icon.ico"
)
foreach ($relativePath in $requiredIcons) {
  if (-not (Test-Path -LiteralPath (Join-Path $releaseRoot $relativePath))) {
    throw "Required release icon is missing: $relativePath"
  }
}

Push-Location $releaseRoot
try {
  & node "scripts/check-version.mjs"
  if ($LASTEXITCODE -ne 0) {
    throw "Version consistency check failed."
  }
} finally {
  Pop-Location
}

Write-Host "[release-preflight] Node/Rust toolchains available"
Write-Host "[release-preflight] MICROSOFT_CLIENT_ID configured: yes"
Write-Host "[release-preflight] tenant configured: yes"
Write-Host "[release-preflight] release icons present"
