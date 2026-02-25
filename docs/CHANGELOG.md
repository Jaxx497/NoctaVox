# CHANGELOG

## Version 0.2.3

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

## Version 0.2.2
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

## Version 0.2.1
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

