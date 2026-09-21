$ErrorActionPreference = "Stop"

$releaseRoot = Split-Path -Parent $PSScriptRoot
$packageJson = Get-Content -LiteralPath (Join-Path $releaseRoot "package.json") -Raw | ConvertFrom-Json
$tauriConfig = Get-Content -LiteralPath (Join-Path $releaseRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$targetRoot = Join-Path $releaseRoot "src-tauri/target/release"
$bundleRoot = Join-Path $targetRoot "bundle"
$binary = Join-Path $targetRoot "ms-todo-desktop-widget.exe"
$expectedMsi = Join-Path $bundleRoot "msi/$($tauriConfig.productName)_$($packageJson.version)_x64_en-US.msi"
$expectedNsis = Join-Path $bundleRoot "nsis/$($tauriConfig.productName)_$($packageJson.version)_x64-setup.exe"

foreach ($artifact in @($binary, $expectedMsi, $expectedNsis)) {
  if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) {
    throw "Release artifact is missing: $artifact"
  }
  if ((Get-Item -LiteralPath $artifact).Length -le 0) {
    throw "Release artifact is empty: $artifact"
  }
}

$binaryInfo = (Get-Item -LiteralPath $binary).VersionInfo
if (-not $binaryInfo.ProductVersion.StartsWith($packageJson.version)) {
  throw "Executable ProductVersion '$($binaryInfo.ProductVersion)' does not match '$($packageJson.version)'."
}
if ($binaryInfo.ProductName -ne $tauriConfig.productName) {
  throw "Executable ProductName '$($binaryInfo.ProductName)' does not match '$($tauriConfig.productName)'."
}

Write-Host "[release-smoke] EXE metadata matches version $($packageJson.version)"
Write-Host "[release-smoke] MSI present: $expectedMsi"
Write-Host "[release-smoke] NSIS present: $expectedNsis"
