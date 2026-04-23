<div align="center">
<h1>NoctaVox
</h1>

[![Crates.io Version](https://img.shields.io/crates/v/noctavox?labelColor=444&color=5555ff)](https://crates.io/crates/noctavox)  [![](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=444&color=fff)](https://ratatui.rs)

**A lightweight, Rust-powered music player designed for local libraries and terminal workflows** </div>

-----

![noctavox.gif](./docs/header.gif)

## Features

- Gapless playback
- Multi-format audio ```mp3, m4a, wav, flac, ogg, opus```
- Live library reloading
- Custom theming with hot reload
- Vim-like key-bindings
- Minimal view mode
- Waveform, oscilloscope, and spectrum visualizations
- Seemless integration with system media controls
- Queue support
- Playlist management

## Installation

**Cargo (recommended)**
```bash
cargo install NoctaVox --locked 
```

**Build With Rust Toolchain**

```bash
git clone https://github.com/Jaxx497/noctavox/
cd noctavox

# Run directly (use the release flag for best audio experience)
cargo run --release 

# Or install globally
cargo install --path noctavox

# and run with the following command:
vox
```

## Quick Start

On first launch, you'll be prompted to set root directories for your music
library. Access this menu anytime with the `` ` `` or `~` key.

**Navigation (Scrolling):** `j` `k` or vertical arrow keys  
**Navigation (Panes):** `h` `l` or horizontal arrow keys  
**Playback:** `Space` to toggle playback, `Enter` to play  
**Seeking:** `n` +5 secs, `p` -5 secs  
**Search:** `/`  
**Add to queue**: `q`  
**Reload library:** `F5` or `Ctrl`+`u`  
**Reload theme:** `F6`  
**Toggle minimal mode:** `m`

See the complete [keymap documentation](./docs/keymaps.md) for much more

## Theming

NoctaVox supports custom themes. The most recent specification for the
theming engine can be found by refering to the [theme
specification](./docs/themes.md). 

Pre-made themes can be installed with the `install-theme` script.

**Linux**
```bash
# from the repo root
chmod +x ./install-themes.sh
./install-themes.sh
```

**Windows Powershell**

```powershell
# from the repo root
./install-themes.ps1
``` 

## About

Supported formats: `mp3`, `m4a`, `wav`, `flac`, `ogg`, `opus` \
Container formats are **not** currently supported: (e.g. `webm`, `mkv`, etc.)

FFmpeg is an optional dependency which enables the waveform visualization
functionality. Without ffmpeg, the functionality will simply fallback onto a
different visualization method.

NoctaVox never over-writes user files and does not have any online
capabilities. The program does however rely on accurate tagging, but does not
supply a method for editting tags. It's strongly recommended that users ensure
their libraries are properly tagged.

> **Tip:** NoctaVox supports hot reloading by pressing `Ctrl+u` or `F5` at any
> point during runtime. The reload will reflect updated metadata, new
> additions, and removals, without needing to restart the runtime.

## Voxio Backend 

In order for NoctaVox to recognize the intended vision without compromise, a
custom backend was written- Voxio. It's an extremely simple audio playback
engine designed to play audio at the highest quality, while also supporting the
OPUS filetype and gapless playback; features that have proven hard to come by
in more mature projects. This backend is being actively developed to increase
user satisfaction and reduce decoding errors. 

As of version 0.2.6, Voxio has been moved into it's own repository, feel free
to view it here: \
https://github.com/Jaxx497/Voxio

## Roadmap 

- Implement cleaner visual design mimimal mode
- Additional user config options (framerate, backend selection??)
- Expanded format testing
- Playlist import/export functionality

## Other

NoctaVox is a hobby project primary written primarily for educational purposes.
This project seeks to demonstrate an understanding of a variety of programming
fundamentals, including but not limited to multi-threading, atomics, string
interning, database integration, de/serialization, memory management, integrity
hashing, session persistence, OS operations, modular design, view models, state
management, user customization, cross-platform development and much more. 

