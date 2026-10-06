param([Parameter(Mandatory = $true)][string]$Out)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$conf = Get-Content (Join-Path $PSScriptRoot "tauri.conf.json") -Raw | ConvertFrom-Json
$version = $conf.version
$repo = $conf.plugins.updater.endpoints[0] -replace "^https://github\.com/", "" -replace "/releases/.*$", ""
$exe = Get-ChildItem -Path $Out -Filter "*-setup.exe" | Select-Object -First 1
if (-not $exe) { exit 1 }
$sigPath = $exe.FullName + ".sig"
if (-not (Test-Path $sigPath)) { exit 1 }
$sig = (Get-Content $sigPath -Raw).Trim()
$asset = [uri]::EscapeDataString($exe.Name) -replace "%20", "."
$json = [ordered]@{
  version  = $version
  notes    = ""
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = $sig
      url       = "https://github.com/$repo/releases/download/v$version/$asset"
    }
  }
}
[IO.File]::WriteAllText((Join-Path $Out "latest.json"), ($json | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
Write-Host "      latest.json -> $(Join-Path $Out 'latest.json')"
