# CHANGELOG

## CURRENT DEVELOPMENT CHANGES

### Added:
  - Repeat/Looping Mode 
    - Toggle repeat with `ctrl`+`r`

### Changed:
  - Updated Voxio to version 0.1.6
    - Voxio now exposes `clear_next` method

### Fixed:
  - Ivoking the GoToAlbum command (`ctrl + A`) on empty table will fallback to
    the sidebar view rather than throwing an error

## [0.2.8] Addons & Config Added

> #### 2026-05-30

### BREAKING CHANGES:
 - Metadata backend rewrote with symphonia instead of lofty
 - Bitrate no longer recorded
   - Still in database to avoid totally breaking existing configs

### Added:
 - Experimental webm support! 
   - Bumped voxio to 0.1.5 for partial webm seek support
 - Addon support!
   - Playlist import/export
   - Now_playing database view
 - CLI Flags
   - no flag launches NoctaVox
   - import (if nv-transpose is installed)
   - export (if nv-transpose is installed)
   - list (if nv-transpose is installed)
   - reset (Deletes database completely)
 - New User Configuration System
   - Control framerate [default: 60fps] `20 <= FR <= 360`
   - Set custom history size
   - Flag to update on startup
   - Auto-resume where last song left off [default: off]
   - Broadcast functionality 
 - Remembers what was playing on last shutdown
    - Do not auto-increment play_count
 - Added several custom themes to the [theme examples folder]( ./docs/theme_examples/ )
 - Improved theme installation script

### Fixed:
 - **[BREAKING?]** Reworked metadata reading pipeline
   - Replaced lofty with symphonia
   - This won't *really* affect anyone but the logic may lead to different
     readings than before on library scans. 
 - `Q` binding restored 
 - Keybinds are more consistent cross-platform
 - Reworked internal timing system
 - Reworked history tracking
    - Fixed bug where songs conditionally wouldn't be added to history
    - Better error handling when database is corrupt
 - Better error handling when Voxio backend fails to start
 - **[minimal mode]** no longer crashes when terminal height < 3 
 - **[minimal mode]** stats window matches theme bg
 - **[minimal mode]** tracklist duration styling normalized 
 - Stats panel won't crash if 0 songs have been played
 - Better toggle playback logic, should enhance media control usability
 - No more hanging threads on shutdown 
   - Fixes hangs on linux systems

### Other: 
 - Added config documentation to [README](./README.md) 
 - Added FAQ section to [README](./README.md)
 - Added new testing profiles

## [0.2.7] Polished Minimal Mode + Many Bug Fixes 

> #### 2026-04-27

### Added: 
 - New breadcrumb widget for simplified minimal mode navigation
 - Cycle through progress widgets with `w`
 - Search engine treats accented characters as equivalents
   - (Ex: `í` and `i` are not differentiated)

### Fixed:
 - Duration values >= 1 hour no longer truncated
   - Fixed for both timer widget and table display
 - Minimal mode padding uses adaptive spacing
 - Shuffle commands no longer overriden
 - Fixed broken key binds on non-Windows platforms  
    - `<`, `>`, `{`, `}`, `?`, `~`
 - Fixed conditional error in `./install-themes.sh`

### Other: 
 - LICENSE file moved to root
 - Playback widgets be set with capitals: `B`, `W`, `O`, `S`
 - Timer placement adjusted
 - Indexing colors are more consistent
 - Songs are verically centered in traditional view
 - Lots of formatting tweaks
 - Search window cleaner in minimal mode
 - Changes to readme
 - Clarifications in theme specification

## [0.2.6] 

> #### 2026-04-15


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

