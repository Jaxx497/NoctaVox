$ThemeDir = Join-Path $env:APPDATA "noctavox\themes"

New-Item -ItemType Directory -Force -Path $ThemeDir | Out-Null
Copy-Item "docs\theme_examples\*.toml" -Destination $ThemeDir

Write-Host "Installed themes to $ThemeDir"
