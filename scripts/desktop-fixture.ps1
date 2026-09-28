param([switch]$Launch)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$fixtureRoot = Join-Path $projectRoot '.test-data\desktop'
New-Item -ItemType Directory -Force -Path $fixtureRoot | Out-Null
$steamRoot = Join-Path $fixtureRoot 'Steam'
$epicRoot = Join-Path $fixtureRoot 'EpicManifests'
$sunshineRoot = Join-Path $fixtureRoot 'Sunshine'
$dataRoot = Join-Path $fixtureRoot 'AppData'
foreach ($path in @($steamRoot, $epicRoot, $sunshineRoot, $dataRoot)) { New-Item -ItemType Directory -Force -Path $path | Out-Null }
foreach ($game in @(@('1091500', 'Cyberpunk 2077'), @('1245620', 'ELDEN RING'), @('1243810', 'Citizen Sleeper'), @('1145360', 'Hades'))) {
    $id = $game[0]
    $name = $game[1]
    New-Item -ItemType Directory -Force -Path (Join-Path $steamRoot "steamapps\common\$name") | Out-Null
    $manifest = '"AppState" { "appid" "' + $id + '" "name" "' + $name + '" "StateFlags" "4" "installdir" "' + $name + '" "LastUpdated" "1750000000" "SizeOnDisk" "500000" }'
    Set-Content -LiteralPath (Join-Path $steamRoot "steamapps\appmanifest_$id.acf") -Value $manifest -Encoding utf8
}
foreach ($game in @(@('AlanWake2', 'Alan Wake 2'), @('Satisfactory', 'Satisfactory'))) {
    $install = Join-Path $fixtureRoot ('Games\' + $game[0])
    New-Item -ItemType Directory -Force -Path $install | Out-Null
    $manifest = @{ AppName = $game[0]; DisplayName = $game[1]; InstallLocation = $install; LaunchExecutable = 'Game.exe'; CatalogNamespace = 'fixture'; CatalogItemId = $game[0]; AppCategories = @('games'); bIsApplication = $true; bIsExecutable = $true }
    $manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $epicRoot ($game[0] + '.item')) -Encoding utf8
}
$appsPath = Join-Path $sunshineRoot 'apps.json'
if (-not (Test-Path -LiteralPath $appsPath)) { Copy-Item -LiteralPath (Join-Path $projectRoot 'src-tauri\tests\fixtures\apps.json') -Destination $appsPath }
$config = @"
[general]
auto_sync = false
start_with_windows = false
start_minimized = false
close_behavior = "exit"
theme = "light"
[providers.steam]
enabled = true
path = '$steamRoot'
[providers.epic]
enabled = true
path = '$epicRoot'
[sunshine]
install_path = '$sunshineRoot'
apps_path = '$appsPath'
reload_mode = "none"
reload_command = ""
[network]
proxy_mode = "direct"
proxy_url = ""
[artwork]
provider = "local"
steamgriddb_api_key = ""
"@
[System.IO.File]::WriteAllText((Join-Path $dataRoot 'config.toml'), $config)
Write-Output "Fixture data: $fixtureRoot"
if ($Launch) {
    $env:SUNSHINE_LIBRARY_SYNC_DATA = $dataRoot
    $executable = Join-Path $projectRoot 'src-tauri\target\release\sunshine-library-sync.exe'
    if (-not (Test-Path -LiteralPath $executable)) { throw 'Build the release executable first with pnpm tauri build --no-bundle.' }
    $process = Start-Process -FilePath $executable -WorkingDirectory $projectRoot -WindowStyle Hidden -PassThru
    Write-Output "Launched fixture process: $($process.Id)"
}
