# Noctavox Theming

> **Specification Version: v0.8**

This document describes the full Noctavox theme specification.

*Example themes can be found in the [theme examples](./theme_examples/)
folder.*

### Quick Theme Example

Here's an example of a take on the Gruvbox Dark theme. This is the simplest
example of a custom theme as all required fields have values. 

```toml
[colors]
surface_global      = "#1e1e2e"
surface_active      = "#282838"
surface_inactive    = "#1a1a26"
surface_error       = "#ff5555"

text_primary        = "#ffffff"
text_secondary      = "#f5c542"
text_secondary_in   = "#b38a2e"
text_selection      = "#1e1e2e"
text_muted          = "#888888"

border_active       = "#f5c542"
border_inactive     = "#444444"

accent              = "#f5c542"
accent_inactive     = "#b38a2e"
```
> Figure 1: `GruvboxDark.toml`

The above is not only the simplest theme, but it encompasses the entirety of
the `[colors]` section. Each value laid out above is required and any omission
will result in an illegal theme (not read in by the engine).

Everything that follows in this document describes optional customization.

## Hotkeys

Several shortcuts are available for working with themes while the
application is running.

| Key | Action |
|---|---|
| Ctrl+C | Open the theme menu |
| F6 | Reload themes from disk |
| Shift + < / > | Cycle through loaded themes |

Note that cycling themes does not reload files that have changed on disk.
After editing a theme, press **F6** to refresh them.

## Theme Location & Properties

Theme files should be placed in: `$CONFIG/noctavox/themes/`

This directory will be created automatically on first launch if it
does not already exist.

Theme files must be valid TOML and contain the `.toml` file extension. The
theme name is derived from the base of the filename.

Themes that fail to parse are skipped silently. Common causes include:

- missing required fields
- typos in field names
- missing `#` in hex colors
- using `:` instead of `=`
- forgetting quotes around strings (like hex values)

The order of sections and fields is irrelevant

--------------------------------------------------

## Theme Structure

A theme file may contain the following sections:
```toml
# Required
[colors] 

# Optional
[borders]                   # Controls border type/visibility
[progress]                  # Global settings for progress widgets
[progress.bar]              # Settings for progress bar         (overrides [progress])
[progress.waveform]         # Settings for waveform widget      (overrides [progress])
[progress.oscilloscope]     # Settings for oscilloscope widget  (overrides [progress])
[progress.spectrum]         # Settings for spectrum widget      (overrides [progress])

[extras]                    # Miscellaneous settings
```

--------------------------------------------------

#### [colors]

This section defines the base color palette used throughout the
application. All fields must be provided.

| Field | Type | Description |
|---|---|---|
| surface_global | [Color](#colors-and-gradients) | Background of the application |
| surface_active | [Color](#colors-and-gradients) | Background of the focused pane |
| surface_inactive | [Color](#colors-and-gradients) | Background of unfocused panes |
| surface_error | [Color](#colors-and-gradients) | Background used for error popups |
| text_primary | [Color](#colors-and-gradients) | Primary text color |
| text_secondary | [Color](#colors-and-gradients) | Highlighted or accented text |
| text_secondary_in | [Color](#colors-and-gradients) | Secondary text in inactive panes |
| text_selection | [Color](#colors-and-gradients) | Text shown inside selections |
| text_muted | [Color](#colors-and-gradients) | Muted or de-emphasized UI text |
| border_active | [Color](#colors-and-gradients) | Border color for focused panes |
| border_inactive | [Color](#colors-and-gradients) | Border color for unfocused panes |
| accent | [Color](#colors-and-gradients) | General accent color used across the UI |
| accent_inactive | [Color](#colors-and-gradients) | Accent color for inactive panes |

--------------------------------------------------

### Progress Widgets

Progress widgets are those seen at the bottom of the NoctaVox window (toggled
via the `w` key). The `[progress]` field provides defaults for playback
visualization widgets. Individual widget attributes override these values
whereas missing or omitted values will inherit from them.

#### [progress]

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | [colors].accent | Color or gradient of widget |
| style | [ProgressStyle](#progress-styles) | dots | Canvas render style for spectrum, waveform, and oscilloscope widgets. |
| speed | float | widget specific | Gradient animation speed |


> *Tip:* Speed values control how fast a gradient scrolls across a widget.
>
>- A value of `0.0` will result in no animation. For the bar, a value of `0.0`
>  will deactivate the gradient and only display the first color.
>- Negative values will reverse the direction of the gradient scroll.
>- Multiples of `2.0` enable the smoothest visual effect when seeking with the
>  waveform widget visible.
>- The progress bar does not have a scrolling effect, but rather a strobing
>  effect
>

Example:

```toml
[progress]
color = ["#ff0000","#ffffff","#0000ff"]
style = "dots"
speed = 6.0
```

#### [progress.bar]

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | progress.color | Played portion color |
| color_unplayed | [InactiveColor](#inactive-color-values) | dimmed | Remaining progress |
| symbol_played | string | `━` | Played character |
| symbol_unplayed | string | `─` | Unplayed character |
| speed | float | 0.0 | Gradient animation |

Example:

```toml
[progress.bar]
symbol_played = "▰"
symbol_unplayed = "▱"
```

#### [progress.waveform]

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | progress.color | Waveform color |
| color_unplayed | [InactiveColor](#inactive-color-values) | dimmed | Unplayed region |
| speed | float | 4.0 | Gradient animation |

> **Tip:** Definitions and examples for *Gradient* and *InactiveColor* are
> discussed at the bottom of this document.

#### [progress.oscilloscope]

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | progress.color | Line color |
| speed | float | 0.0 | Gradient animation |


#### [progress.spectrum]

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | progress.color | Bar color |
| mirror | bool | false | When `true`, mirrors the spectrum horizontally.|
| decay | float | 0.85 | How quickly spectrum bars fall after a peak. Clamped between `0.7` and `0.97`. Higher values = slower decay. |
| speed | float | 0.0 | Gradient animation |


--------------------------------------------------

#### [borders]

| Field | Type | Default | Description |
|---|---|---|---|
| display | bool | true | Enable borders |
| style | [BorderStyle](#border-styles) | rounded | Border style |


> **Tip:** Definition for *BorderStyle* is discussed at the bottom of this
> document.

--------------------------------------------------

#### [extras]

| Field | Type | Default | Description |
|---|---|---|---|
| is_dark | bool | true | Dark/light hint |
| decorator | string | ✧ | Decorative glyph |

> **Tip:** The `is_dark` field tells the engine if the theme is considered to be a dark
theme or light theme. This affects how colors are dimmed.

--------------------------------------------------

#### Colors and Gradients
    
Acceptable color formats include the following:

**Hex:** `"#1a2b3c"`  
**RGB:** `"rgb(255,50,120)"`  

> **Note: DON'T FORGET THE QUOTATION MARKS!**

The gradient type enables users to use multiple colors to display a widget, put
together into a gradient. Simply define the colors inside of a pair of
brackets. Gradients are not required, and users can opt for individual colors:

```toml
# Both of these are legal values for anything that expects the GRADIENT type
color = "#ff00ff"   
color = ["#ff0000","#ffffff","#0000ff"]
```

#### Transparency 
Noctavox supports transparent values for many fields. To set a color as
transparent, simply provide an empty pair of quotation marks: `""` or fill them
with the word `"none"`  
As a general disclaimer, this won't work on every field and is
somewhat dependent on your terminal emulator.

--------------------------------------------------

#### Inactive Color Values

The `InactiveColor` type controls how unplayed portions of widget is colored. 

| Value | Description |
|---|---|
| `"dimmed"` | The active gradient, darkened based on `is_dark` and audio amplitude. |
| `"still"` | A frozen (non-animated) version of the active gradient, also darkened. |
| `"#rrggbb"` / `"rgb(...)"` | A solid single color. |
| `["#...", "#...", ...]` | An independent gradient, unrelated to the active color. |

--------------------------------------------------

#### Progress Styles

Used by waveform and oscilloscope.

    "dots"
    "block2" (alias: halfblock)
    "block4" (alias: quadrant)
    "block6" (alias: sextant)
    "block8" (aliases: octant, blocks)

--------------------------------------------------

#### **Border Styles**
```Toml

    "Plain"
    "Rounded"   #default
    "Double"
    "Thick"
    "LightDoubleDashed"
    "HeavyDoubleDashed"
    "LightTripleDashed"
    "HeavyTripleDashed"
    "LightQuadrupleDashed"
    "HeavyQuadrupleDashed"
    "QuadrantInside"
    "QuadrantOutside"
```

Invalid values fall back to Rounded.
