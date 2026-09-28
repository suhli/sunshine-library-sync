param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')]
    [string]$Version,
    [string]$OutputDir = 'release'
)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$exe = Join-Path $projectRoot 'src-tauri/target/release/sunshine-library-sync.exe'
$installer = Join-Path $projectRoot "src-tauri/target/release/bundle/nsis/Sunshine Library Sync_${Version}_x64-setup.exe"
$readme = Join-Path $projectRoot 'README.md'
foreach ($file in @($exe, $installer, $readme)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
        throw "Release input is missing: $file"
    }
}

if (-not [IO.Path]::IsPathRooted($OutputDir)) {
    $OutputDir = Join-Path $projectRoot $OutputDir
}
$OutputDir = [IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$baseName = "Sunshine-Library-Sync_v${Version}_win10-win11_x64"
$zip = Join-Path $OutputDir "${baseName}_portable.zip"
$nsis = Join-Path $OutputDir "${baseName}_nsis.exe"
Compress-Archive -LiteralPath @($exe, $readme) -DestinationPath $zip -CompressionLevel Optimal -Force
Copy-Item -LiteralPath $installer -Destination $nsis -Force
Write-Output $zip
Write-Output $nsis
