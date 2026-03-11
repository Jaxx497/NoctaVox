# Noctavox Theming

> Specification Version: v0.8

This document describes the full Noctavox theme specification. Fields not defined are ignored by the parser. Example themes can be found in: [theme examples](./theme_examples/)


### Quick Theme Example

The smallest usable theme looks like this:

```toml
# Minimal Gruvbox

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

Only the `[colors]` section is required.
Everything else in this document describes optional customization.

### Hotkeys

Several shortcuts are available for working with themes while the
application is running.

| Key | Action |
|---|---|
| Ctrl+C | Open the theme menu |
| F6 | Reload themes from disk |
| Shift + < / > | Cycle through loaded themes |

Note that cycling themes does not reload files that have changed on disk.
After editing a theme, press **F6** to refresh them.

### Theme Location & Properties

Theme files should be placed in: `$CONFIG/noctavox/themes/`

This directory will be created automatically on first launch if it
does not already exist.

Theme files must be valid TOML and contain the `.toml` file extension. The
theme name is derived from the filename (minus the extension).

Themes that fail to parse are skipped silently. Common causes include:

- missing required fields
- typos in field names
- missing `#` in hex colors
- using `:` instead of `=`
- forgetting quotes around strings

The order of sections and fields is irrelevant

--------------------------------------------------

### Theme Structure

A theme file may contain the following sections:

    [colors]                (required)

    [borders]
    [progress]              (optional)
    [progress.bar]
    [progress.waveform]
    [progress.oscilloscope]
    [progress.spectrum]

    [extras]

Only the `[colors]` section is mandatory. All other sections provide
additional customization for specific UI components.

--------------------------------------------------

#### [colors]

This section defines the base color palette used throughout the
application. All fields listed below must be provided.

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

Provides defaults for playback visualizations.

Widgets:

    [progress.bar]
    [progress.waveform]
    [progress.oscilloscope]
    [progress.spectrum]

Widget attributes override values of the same name in `[progress]`. Missing
values inherit from it.

#### [progress]
Global Fields

| Field | Type | Default | Description |
|---|---|---|---|
| color | [Gradient](#colors-and-gradients) | accent | Color or gradient of widget |
| style | [ProgressStyle](#progress-styles) | dots | Canvas render style for spectrum, waveform, and oscilloscope widgets. |
| speed | float | widget specific | Gradient animation speed |


> *Tip:* Speed values control how fast a gradient scrolls across a widget.
>
>- A value of `0.0` will result in no animation. For the bar, a value of `0.0`
>  will deactivate the gradient and only display the first color.
>- Negative values will reverse the direction of the gradient scroll.
>- Multiples of `2.0` enable the smoothest visual effect when seeking with the
>  waveform widget visible.
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

--------------------------------------------------

#### [extras]

| Field | Type | Default | Description |
|---|---|---|---|
| is_dark | bool | true | Dark/light hint |
| decorator | string | ✧ | Decorative glyph |

--------------------------------------------------

#### Colors and Gradients
    
Acceptable color formats include the following:

    Hex: "#1a2b3c"
    RGB: "rgb(255,50,120)"
    Transparent: "" or "none"

> **Note:** Always remember quotation marks!

Any type that takes a gradient type will also allow a single color to be
entered. Both of the following would be legal values:

```toml
color = "#ff00ff"
color = ["#ff0000","#ffffff","#0000ff"]
```

--------------------------------------------------

#### Inactive Color Values

Used by `color_unplayed` fields. Controls how the unplayed portion of a
widget is rendered.

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
