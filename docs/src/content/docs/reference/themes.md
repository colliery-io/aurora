---
title: Themes
description: The Aurora themes, the data-theme attribute, the storage key and the theme functions.
sidebar:
  order: 4
---

The theme code is in `aurora_leptos::theme` (also at the crate root).

## The themes

The `data-theme` attribute on `<html>` sets the theme. `color-scheme` tells
the browser which value of each `light-dark()` token to use.

| Theme | `Theme` | `data-theme` | `color-scheme` | Needs |
|---|---|---|---|---|
| System | `Theme::System` (the default) | none | `light dark`: follows the OS, live | — |
| Light | `Theme::Light` | `light` | `light` | — |
| Dark | `Theme::Dark` | `dark` | `dark` | — |
| Hyper | `Theme::Hyper` | `hyper` | `dark`, from the hyper stylesheet | `HYPER_CSS` on the page |

With no hyper stylesheet, `data-theme="hyper"` matches no rule, and the page
follows the OS.

## Storage

| | |
|---|---|
| Key | `aurora-theme` (`THEME_STORAGE_KEY`) |
| Where | `localStorage` |
| Values | `"light"`, `"dark"`, `"hyper"`. No key means System. |

If storage is not there, or it throws, the page follows the OS and shows no
error.

## THEME_INIT_SCRIPT

Put this script in an inline `<script>` in the `<head>`, before the
stylesheet. It copies the stored choice to `data-theme` before the first
paint. For light and dark, it also sets the inline `color-scheme`.

```html
<meta name="color-scheme" content="light dark" />
<script>(function(){try{var t=localStorage.getItem("aurora-theme");if(t==="light"||t==="dark"||t==="hyper"){var d=document.documentElement;d.setAttribute("data-theme",t);if(t!=="hyper")d.style.colorScheme=t;}}catch(e){}})();</script>
```

This is the script of 0.5. The script of 0.4 does not know `"hyper"`.

## The Theme type

| Method | Gives |
|---|---|
| `as_str()` | `"light"`, `"dark"`, `"hyper"` or `"system"` |
| `parse(text)` | The theme of a stored value, or `None` |
| `label()` | `"Light"`, `"Dark"`, `"Hyper"`, `"System"` |
| `is_dark_base()` | `true` for Dark and Hyper |

`toggle_choices(hyper)` gives the buttons of `ThemeToggle`, in order:
Light, Dark, (Hyper,) System.

## Functions

These need the `components` feature (the default).

| Function | What it does |
|---|---|
| `set_theme(theme)` | Shows `theme` and stores it. `System` removes the stored choice. |
| `apply_theme(theme)` | Shows `theme`, and does not store it. |
| `current_theme()` | The theme on `<html>` now. Hyper with no hyper stylesheet is `System`. |
| `stored_theme()` | The stored theme, or `System`. Hyper with no hyper stylesheet is `System`. |
| `system_prefers_dark()` | `true` if the OS asks for dark. |
| `hyper_available()` | `true` if the hyper stylesheet is on the page. |
| `provide_theme()` | Makes a `ThemeContext` and provides it. Call it once, at the root. |
| `use_theme()` | The `ThemeContext`. It makes one if there is none. |

## ThemeContext

| Member | What it is |
|---|---|
| `choice` | `RwSignal<Theme>`: the choice. |
| `system_dark` | `RwSignal<bool>`: `true` while the OS asks for dark. It changes live. |
| `set(theme)` | Changes the theme, stores it, and updates `choice`. |
| `theme()` | The choice (reactive). |
| `is_dark()` | `true` if the page has a dark base now: Dark, Hyper, or System on a dark OS (reactive). |

## ThemeToggle

`<ThemeToggle />` shows Light, Dark and System. `<ThemeToggle hyper=true />`
adds Hyper. Each button has `aria-pressed`. See
[Components](../components/#themetoggle).
