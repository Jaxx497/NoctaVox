# CHANGELOG

## [0.2.7]

### Added: 
 - New breadcrumb widget for simplified minimal mode navigation
 - Cycle through progress widgets with `w`

### Fixed:
 - Enhanced search engine to handle combining marks  
    - (Ex: `í` and `i` are not differentiated)
 - Shuffle commands no longer overriden
 - Fixed broken key binds on non-Windows platforms  
    - `<`, `>`, `{`, `}`, `?` 
 - Fixed conditional error in `./install-themes.sh`

### Other: 
 - Playback widgets must be set with capitals: `B`, `W`, `O`, `S`
 - Lots of formatting changes
 - Search window cleaner in minimal mode
 - Updated docs

## [0.2.6] - 2026-04-15


### New Features 
 - NoctaVox now reports to OS media controls

### Fixed
 - Fixed voxio seek errors on mp3 files
 - Fixed visual bugs when searching in mimimal mode
 - Fixed inconsistent multi-select behavior
 - Several minimal mode formatting fixes

### Other
 - Voxio moved to its own repository: 
    https://github.com/Jaxx497/Voxio
 - Bumped ratatui-textarea to version 0.9

## [0.2.5]

MINIMAL MODE BETA

 - Timer re-enabled
 - Enter minimal mode with `m` keybinding
 - Multi-select count now in border of main window
 - Spectrum widget freezes on pause instead of slowly draining
 - Oscilloscope operates at a lower resolution, making it visually cleaner
 - Bufferline info overlaps playback widgets instead of having a dedicated row
 - Song titles get more space allocated in bufferline
 - Widgets now have reactive size elements

 - Fixed bug where numbers could not be entered into text fields

## [0.2.4]

NEW THEME SPECIFICATION* v0.8

Optimized startup logic (skip disk read if no changes detected)
Close fullscreen when queue and playback are empty
Non-bar widgets responsive sizing depending on window height

*Theme info
  - All fields outside of the [colors] section are completely optional
  - Selection field merged into `accent`
    - (Respective `inactive` field also merged)
  - Progress section overrides default values
  - Fine tune specific widgets with `progress.[identifier]` tag

Added theme installation scripts

## [0.2.3]

Added spectrum-analyzer widget

User statistics can now be displayed via `?`
Voxio sample and tap no longer push on a per sample basis, but rather in chunks
Voxio should have less data races
Voxio exposes channels and sample_rate via public API

New maps:
 - `=` Go to album-view of the currently playing track
 - `?` View library and listening statistics
 - `s` Spectrum view
 - `S` Spectrum view [full screen]

Switched `Alt`+`1`, `Alt`+`2`, `Alt`+`3` to be `Ctrl`+`1`, `Ctrl`+`2`, `Ctrl`+`3`

## [0.2.2]
Licensing added

Voxio is now available on crates.io \
Voxio should not report active until verifying a single valid packet \
Voxio no longer prints to screen when errors occur in the main callback

Numeric command prefixing has been implemented for scrolling, multi-selection,
and playback. Review the instructions in [the keymaps
documentation](./keymaps.md) for more information.

**`1`, `2`, `3` no longer map to Album/Playlist/Queue views respectively** \
These maps have been replaced with `Alt`+`1`, `Alt`+`2`, `Alt`+`3` \
Consider using the standard `Ctrl`+`A`, `Ctrl`+`T`, `Ctrl`+`Q` maps instead


Minor visual bugs have been resolved, including extreme strobing from progress
widgets

## [0.2.1]
Voxio is now the default backend.

Crossbeam has been integrated. All event driven
architecture now uses bounded crossbeam channels, and all
events are handled by the `select!` macro for increased
responsiveness. Furthermore, the crossbeam ArrayQueue
removes the need for any lock-based architecture within the
project.

Main loop and library initialization logic has been cleaned
up substantially.

Error handling throughout the program has been seriously
buffed.

A single FFMPEG check is made on intialization rather than
everytime a waveform is generated.

Small visual tweeks

Updated docs

New dependencies: 
- Voxio
- Crossbeam (channel and queue)

Removed dependencies:
- Parking lot
- Rodio

