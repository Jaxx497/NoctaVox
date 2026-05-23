# Download Noctavox theme examples directly from GitHub.
# Usage:  irm https://raw.githubusercontent.com/Jaxx497/NoctaVox/master/get-themes.ps1 | iex

$ErrorActionPreference = "Stop"

$Owner    = "Jaxx497"
$Repo     = "NoctaVox"
$Branch   = "master"
$ThemeDir = if ($env:NOCTAVOX_THEME_DIR) { $env:NOCTAVOX_THEME_DIR } else { Join-Path $env:APPDATA "noctavox\themes" }

$ApiUrl = "https://api.github.com/repos/$Owner/$Repo/contents/docs/theme_examples?ref=$Branch"

New-Item -ItemType Directory -Force -Path $ThemeDir | Out-Null

try {
    $files = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "noctavox-installer" }
} catch {
    Write-Error "Failed to list themes from $ApiUrl`n$($_.Exception.Message)"
    exit 1
}

$themes = @($files | Where-Object { $_.name -like "*.toml" })

if ($themes.Count -eq 0) {
    Write-Error "No themes found at $ApiUrl"
    exit 1
}

foreach ($file in $themes) {
    $dest = Join-Path $ThemeDir $file.name
    try {
        Invoke-WebRequest -Uri $file.download_url -OutFile $dest -UseBasicParsing -Headers @{ "User-Agent" = "noctavox-installer" }
        Write-Host "  $($file.name)"
    } catch {
        Write-Warning "  ! failed: $($file.name) - $($_.Exception.Message)"
    }
}

Write-Host "Installed $($themes.Count) themes to $ThemeDir"
