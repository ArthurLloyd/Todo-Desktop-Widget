$ErrorActionPreference = "Stop"

$releaseRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "release-preflight.ps1")

function Invoke-ReleaseStep {
  param(
    [string]$Name,
    [string]$Command,
    [string[]]$Arguments
  )

  Write-Host "[release] $Name"
  & $Command @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "$Name failed with exit code $LASTEXITCODE"
  }
}

Push-Location $releaseRoot
try {
  Invoke-ReleaseStep -Name "frontend build" -Command "npm" -Arguments @("run", "build")
  Invoke-ReleaseStep -Name "frontend tests" -Command "npm" -Arguments @("run", "test:frontend")
  Invoke-ReleaseStep -Name "Rust check" -Command "cargo" -Arguments @("check", "--manifest-path", "src-tauri/Cargo.toml")
  Invoke-ReleaseStep -Name "Rust format check" -Command "cargo" -Arguments @("fmt", "--check", "--manifest-path", "src-tauri/Cargo.toml")
  Invoke-ReleaseStep -Name "Rust tests" -Command "cargo" -Arguments @("test", "--manifest-path", "src-tauri/Cargo.toml")
  Invoke-ReleaseStep -Name "Tauri Windows bundles" -Command "npm" -Arguments @("run", "tauri:build")
  Invoke-ReleaseStep -Name "release artifact smoke test" -Command "powershell" -Arguments @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    "scripts/release-smoke-test.ps1"
  )
} finally {
  Pop-Location
}
