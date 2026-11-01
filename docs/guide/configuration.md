# Configuration

iguard reads a single TOML file:

```
$XDG_CONFIG_HOME/iguard/config.toml
```

If `XDG_CONFIG_HOME` is unset or not absolute, it falls back to `~/.config/iguard/config.toml`.

The file is created with defaults on first run. **Changes are picked up automatically** — iguard polls the file's modification time every second and applies edits live, including theme, typography, behavior, and most window settings.

Every key is optional. Omitted keys fall back to the defaults shown below.

## `[appearance]`

Controls colors and theme.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | string | `"dark"` | Built-in theme name (see below). |
| `background` | string? | — | Hex color override. |
| `text` | string? | — | Hex color override. |
| `primary` | string? | — | Hex color override. |
| `success` | string? | — | Hex color override. |
| `warning` | string? | — | Hex color override. |
| `danger` | string? | — | Hex color override. |

**Supported themes:** `dark`, `light`, `dracula`, `nord`, `solarized-light`, `solarized-dark`, `gruvbox-light`, `gruvbox-dark`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `tokyo-night-storm`, `tokyo-night-light`, `kanagawa-wave`, `kanagawa-dragon`, `kanagawa-lotus`, `moonfly`, `nightfly`, `oxocarbon`, `ferra`.

An unknown theme name falls back to `dark`.

**Colors** accept `#RGB`, `#RRGGBB`, or `#RRGGBBAA` (the `#` is optional). Any color you set overrides that slot in the base theme's palette; colors you leave out are inherited from the theme.

```toml
[appearance]
theme = "catppuccin-mocha"
primary = "#89b4fa"
danger = "#f38ba8"
```

## `[window]`

Controls the dialog window.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `width` | float | `520.0` | Window width in logical pixels (clamped 320–4096). |
| `height` | float | `380.0` | Window height in logical pixels (clamped 200–4096). |
| `placement` | enum | `"center"` | `default`, `center`, or `fixed`. |
| `x` | float | `0.0` | X position, only used when `placement = "fixed"`. |
| `y` | float | `0.0` | Y position, only used when `placement = "fixed"`. |
| `resizable` | bool | `false` | Allow the user to resize the window. |
| `decorations` | bool | `true` | Show window decorations (title bar / borders). |
| `always_on_top` | bool | `false` | Keep the dialog above other windows. |
| `closeable` | bool | `true` | Allow the window to be closed. |
| `minimizable` | bool | `false` | Allow minimizing. |

Changing `placement`, `closeable`, or `minimizable` live will **reopen** the window; other changes are applied in place.

```toml
[window]
width = 460.0
height = 340.0
placement = "center"
always_on_top = true
```

## `[behavior]`

Controls interaction.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `autofocus` | bool | `true` | Focus the password field when a prompt appears. |
| `close_on_escape` | bool | `true` | Pressing <kbd>Esc</kbd> cancels authentication. |
| `submit_on_enter` | bool | `true` | Pressing <kbd>Enter</kbd> submits the password. |
| `max_attempts` | integer | `3` | Failed attempts before giving up (clamped 1–10). |
| `show_action_id` | bool | `false` | Show the Polkit action ID in the dialog. |
| `show_details` | bool | `false` | Show Polkit action details in the dialog. |

```toml
[behavior]
max_attempts = 5
show_details = true
show_action_id = true
```

## `[text]`

All UI strings. Useful for translation or rewording.

| Key | Default |
|-----|---------|
| `window_title` | `"iguard"` |
| `title` | `"Authentication required"` |
| `identity` | `"Identity"` |
| `waiting` | `"Waiting for authentication..."` |
| `authenticating` | `"Authenticating..."` |
| `password` | `"Password"` |
| `submit` | `"Authenticate"` |
| `cancel` | `"Cancel"` |
| `failed` | `"Authentication failed"` |
| `cancelled` | `"Authentication cancelled"` |
| `retry` | `"Authentication failed. Try again."` |
| `action` | `"Action"` |
| `details` | `"Details"` |

```toml
[text]
title = "Se requiere autenticación"
submit = "Autenticar"
cancel = "Cancelar"
password = "Contraseña"
```

## `[typography]`

Fonts and spacing.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `font` | enum | `"sans-serif"` | `sans-serif`, `serif`, `monospace`, `cursive`, `fantasy`. |
| `weight` | enum | `"normal"` | `thin`, `extra-light`, `light`, `normal`, `medium`, `semibold`, `bold`, `extra-bold`, `black`. |
| `title_size` | float | `22.0` | Title text size. |
| `body_size` | float | `14.0` | Body text size. |
| `small_size` | float | `13.0` | Small text size. |
| `input_size` | float | `16.0` | Password field text size. |
| `spacing` | float | `10.0` | Space between rows. |
| `padding` | float | `24.0` | Outer window padding. |
| `input_padding` | float | `10.0` | Password field inner padding. |
| `button_padding_x` | float | `16.0` | Horizontal button padding. |
| `button_padding_y` | float | `8.0` | Vertical button padding. |

```toml
[typography]
font = "monospace"
weight = "medium"
title_size = 24.0
body_size = 15.0
```

## Full example

```toml
[appearance]
theme = "nord"
primary = "#88c0d0"

[window]
width = 480.0
height = 360.0
placement = "center"
always_on_top = true

[behavior]
max_attempts = 3
show_details = true
autofocus = true

[text]
title = "Authentication required"
submit = "Authenticate"
cancel = "Cancel"

[typography]
font = "sans-serif"
weight = "normal"
body_size = 14.0
```
