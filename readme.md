<h1 style="text-align: center"> NoctaVox [v0.2.1]

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)
</h1>


NoctaVox is a lightweight, plug and play, TUI music player for local music.

![noctavox.gif](./docs/header.gif)

## Features

- Gapless playback with queue support
- Multi-format audio (mp3, m4a, wav, flac, ogg, opus)
- Live library reloading
- Custom themeing with hot reload
- Vim-like keybindings
- Waveform and oscilloscope visualiztion
- Playlist management

## Installation

```bash
git clone https://github.com/Jaxx497/noctavox/
cd noctavox

# Run directly (use stable for best audio experience)
cargo run --release 

# Or install globally
cargo install --path noctavox_core
noctavox
```

## Quick Start


On first launch, you'll be prompted to set root directories for your music library. Access this menu anytime with `` ` ``.

**Navigation:** `hjkl` or arrow keys  
**Playback:** `Space` to pause, `Enter` to play  
**Seeking:** `n` +5 secs, `p` -5 secs  
**Search:** `/`  
**Reload:** `F5` or `Ctrl+u`  

See [keymaps documentation](./docs/keymaps.md) for the complete list.

## Themeing

NoctaVox supports custom themes. The most recent specification for the
themeing engine can be found by refering to the [themeing
specification](./docs/themes.md). Themes can be live reloaded during
runtime with `F6`. 

## Notes

Supported formats: `mp3`, `m4a`, `wav`, `flac`, `ogg`, `opus`.  
Container formats are not currently supported.

FFmpeg is an optional dependency which enables the waveform visualization functionality.

NoctaVox never writes to user files and does not have any online capabilities.
The program does however rely on accurate tagging, but does not supply any
method for doing so. It's strongly recommended that users ensure their
libraries are properly tagged with a tool like
[MP3Tag](https://www.mp3tag.de/en/) or a similar alternative. 

> **Tip:** NoctaVox supports hot reloading by pressing `Ctrl+u` or `F5` at any
> point during runtime.

## Voxio Backend 

For NoctaVox to recognize its true potential, a custom backend was
written- Voxio. It's extremely simple audio playback engine designed to
play audio at the highest quality, while also supporting the OPUS filetype
and gapless playback; features that have proven hard to come by in more
well known projects. This backend is being actively developed to avoid
increase user satisfaction and reduce decoding faults

## Roadmap 

- Additional user config options (framerate, backend selection)
- Enhanced info display
- Expanded format testing
- Playlist import/export functionality

## Other

NoctaVox is a hobby project primary written for educational purposes. This
project seeks to demonstrate my understanding of a series of programming
fundamentals, including but not limited to multi-threading, atomics, string
interning, database integration, de/serialization, memory management, integrity
hashing, session persistence, OS operations, modular design, view models, 
state management, user customization, and much more. 

