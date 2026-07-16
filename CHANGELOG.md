# CHANGELOG

## UNRELEASED

#### Sidebar Overhaul

The sidebar has undergone a complete overhaul. Navigation between albums and
playlists is now done through foldable headers rather than mode switching. This
simplified the render and state logic significantly.

### Added:
  - Sidebar folding with `enter`/`h`/`l`/`L` navigation keys
  - `expanded` / `collapsed` icon fields to `config.toml` `[icons]` section
  - `expanded` / `collapsed` fields to theme `[icons]` spec (override config)
  - Tracklist scroll padding
  - `NodeKey::Root(Root)` variant consolidating `MusicRoot`/`PlaylistsRoot`

### Changed:
  - Sidebar is now a single unified tree view containing albums and playlists
  - `Ctrl`+`t` now navigates to playlists (`GoToPlaylists`)
  - Enhanced error reporting on invalid config.toml
  - Condensed sidebar state and render logic 
  - `Mode::Library` no longer carries a sub-view 
  - Breadcrumbs unified for all library modes
  - `x` now deletes playlists instead of `Ctrl`+`d`
  - Playlist create/rename/delete rebuilds sidebar rows immediately
  - Snapshot restore now sorts albums on load for consistent ordering
  - Spectrum analyzer uses fewer allocations per frame
  - Filtered out non-existent songs when building playlists from database
  - Various dependency version bumps
  - Various clippy fixes across the codebase

### Removed:
  - `LibraryView` enum (Albums, Playlists) — replaced by tree collapse
  - `SideBarAlbum` and `SideBarPlaylist` widgets — merged into tree handler
  - Separate album/playlist browser input contexts (`InputContext::AlbumView`,
    `InputContext::PlaylistView`) — merged into `InputContext::Sidebar`
  - `Ctrl`+`1`/`Ctrl`+`2`/`Ctrl`+`3`/`Ctrl`+`0` shortcuts
  - Dead imports and unused code

## [0.3.2] - Keymap helper, config changes, volume adjustment
> #### **2026-07-10**

### BREAKING:
**CONFIG.TOML** now has 2 headers **\[general\]** and **\[icons\]**
- All existing config must now include the `[general]` header above it.
- Stats now accessed by `\ (backslash)` key
- `?` now maps to keymap helper

**THEME SPEC**: `[extras]` replaced by `[meta]` and `[icons]`
- `is_dark` is now `dark` under the `[meta]` section
- `decorator` moved to the `[icons]` section
- Old `[extras]` sections are silently ignored — light themes should update
  or they will fall back to `dark = true`

### Added: 
  - Keymap guide popup, accessed with `?`
  - `+` and `-` can now be used to adjust volume (persists sessions)
  - volume meter in traditional view
  - `,` and `.` can now be used to cycle themes 
  - `config.toml` creates itself if it doesn't exist
  - added `[icons]` section to `config.toml`
  - added `[icons]` section to theme spec (override config)
  - added `[meta]` section in theme config
  - More icon control in themes

### Changed:
  - `config.toml` existing settings must sit behind \[general\] header
  - Sidebar max size capped at 49% instead of 40%
  - Reworked live update logic and library refresh process
  - Breadcrumb widget has underlines now
  - Condensed internal action handling
  - Shaved off library clone
  - Minor refactor of UI engine
  - Improved encapsulation acrossed modules
  - Themes moved to its own module
  - Enhanced device recovery logic (from voxio version bump)
  - Crossbeam-channel version bump

## [0.3.1] Voxio Updates
> #### **2026-06-30**  
> *Mostly cosmetic update, following up the Voxio overhaul. Enhanced internal stability, but a new waveform rendering engine and added configuration fields!*


### RESTORED LICENSE FILE
  - Accidentally deleted last commit, restored.

### IN-HOUSE WAVEFORM GENERATION
  - Voxio now exposes a `waveform` feature which builds waveforms using the
    same decode loop for playback. This has replaced the previous *ffmpeg*
    based approach.
  - A consequence of this new waveform generation method is sample accurate
    duration reporting, which will be applied to new and existing databases.
  - **WAVEFORMS WILL HAVE A DIFFERENT SHAPE THAN BEFORE**
    - (Honestly you probably won't even notice)
    - If this bothers you, delete the `waveforms` table from `noctavox.db`. 

    ```bash
    > cd ~/{$CONFIG}/noctavox/ 
    > sqlite3 noctavox.db "DELETE FROM waveforms"

    -- or --

    > vox --reset   # THIS WILL DETELE ALL LISTENING DATA, PLAYLISTS,
                    # PREFERENCES, CACHED DATA, AND OF COURSE, THE
                    # REPRESENTATION OF YOUR LIBRARY 
                    # (user files never written to by NoctaVox)
    ```

### Added:
  - Program title now sets to `NoctaVox` on most platforms
  - UserConfig introduced `seek_small` and `seek_large` fields for customizable
    seek steps

### Removed:
  - Waveform render logic no longer utilizes ffmpeg
  - Removed all syscalls calls to ffmpeg/ffprobe
  - Removed excess waveform enums and structs
  - Removed excess derive traits from various structs
  - Removed nix dependency listing of ffmpeg

### Other Changes:
  - Voxio bumped to version 0.2.1
  - Spectrum analyzer is now the default/fallback display widget
  - Enhanced seek stability
  - Realtime duration updates on poorly tagged tracks
  - Enhanced resilience against corrupt packets
  - Improved error handling
  - `SimpleSong.duration` now of type AtomicU64 
  - Additional bug fixes and stability improvements


## [0.3.0] VOXIO OVERHAUL
> #### 2026-06-26
Voxio has been rewritten from the ground up. A lot of care has gone into the
stability and compatibility of the engine. The API has been cleaned up. The
engine reports events as they happen without the need for polling. OPUS support
is feature gated (although enabled for NoctaVox) and a comprehensive testing
suite acts as a safeguard and guarantee for future changes. This rewrite solves
several bugs, including the obnoxious crash on device switch that plagued
previous versions of NoctaVox.

### Added:
  - Support for ReplayGain via user config
  - Support for AIFF, ALAC, and WEBM*
  - Repeat/Looping Mode (resolves issue #22) 
    - Toggle repeat with `ctrl`+`r`
  - Added MSRV tag (1.95)

  > ***Note:** WEBM support may be slightly unstable depending on metadata

### Voxio Replated Changed:
  > Bumped Voxio to version 0.2.0

  - **Voxio now persists playback on device change** (resolves #3)
  - Voxio now supports ReplayGain (resolves #25)
  - Voxio now uses an event system
  - Voxio now hands the tap off rather than maintaining ownership
  - Voxio now exposes a `clear_next` method
  - Tap system requires significantly less allocations
  - Many more API changes
  - Added a large testing suite

#### **Tighter Voxio Integration**

  - Reworked the PlayerHandle to be a direct fit with Voxio
  - Removed Playercore type
  - Removed PlayerBackend Trait
  - Removed VoxEngine container
  - Removed PlayerMetrics type
  - Removed PlaybackState enum
  - Removed PlayerEvent enum
  - Removed VoxioTrack type
  - Removed QueueDelta enum

### Other Changes:

  - Widgets read state from player directly
  - Underlying queue management system radically simplified
  - Cleaner integration between main thread and audio thread
  - Cleaned up unnecessary allocations
  - Less unwraps & expect statements
  - Opted for more references than clones in various contexts
  - Minor optimizations & enhanced stability 
  - Removed crossbeam::ArrayQueue dependency
  - Removed windows dependency (not to be confused with windows-sys)
  - Cleaned up & removed ratatui serde feature

### Fixed:
  - **Users can change audio device without crashing the audio thread!** (resolves #3)
  - Ivoking the GoToAlbum command (`ctrl + A`) on empty table will fallback to
    the sidebar view rather than throwing an error
  - WAV metadata reading restored (workaround for symphonia upstream bug)
  - Queued track icon color fixed in search view
  - Keybinds between standard view and fullscreen keybinds are more consistent
  - Theme selector now sorts themes the same way between various OS platforms
  - History - Fixed bug where wrong side of Deque would be popped
  - History - fixed broken history count
  - Nix flake updated

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

