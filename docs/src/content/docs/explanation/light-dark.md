---
title: Light-dark tokens and color-scheme
description: Why each Aurora colour token holds two values, and how the browser picks one.
sidebar:
  order: 1
---

Each colour token in Aurora holds its light value and its dark value in one
declaration:

```css
:root {
  --panel: light-dark(#ffffff, #161a21);
}
```

`light-dark()` is a CSS function. The browser gives the first value when the
element's `color-scheme` is light, and the second when it is dark. Aurora sets
`color-scheme` on `<html>`, from the `data-theme` attribute:

```css
:root { color-scheme: light dark; }
:root[data-theme="light"] { color-scheme: light; }
:root[data-theme="dark"] { color-scheme: dark; }
```

With no `data-theme`, `color-scheme` is `light dark`, and the browser follows
the operating system. When the OS setting changes, the page changes, with no
script.

`light-dark()` needs a browser of 2024 or later.

## Why one declaration

Another way is two blocks of tokens: one for light and one for dark, with a
selector for each. Aurora does not do that, for these reasons:

- A token cannot have a light value and lose its dark value. The pair is on
  one line, and the contrast test reads both.
- Each theme comes from one attribute on `<html>`. A component does not know
  the theme. It uses `var(--panel)`, and the value follows.
- `color-scheme` also changes the native parts of the page: scroll bars,
  the pop-up of a `<select>`, date pickers. A page with two blocks of tokens
  must set it too.

## Rust follows too

In Rust, `token::ICE` is the text `"var(--ice)"`, not a colour value. An
inline style with it follows the theme, as the stylesheet does. That is why
you must not add hex alpha to a token: `"var(--ice)1c"` is not a colour. Use
`tint(token::ICE, 11)`, which writes a `color-mix()`.

## No flash

The theme that the reader chose is in `localStorage`. The WebAssembly of the
app loads after the first paint, so it is too late to set `data-theme`.
`THEME_INIT_SCRIPT` is a small inline script for the `<head>`. It sets
`data-theme` before the first paint, so a dark page does not flash light.

## Hyper is not light-dark

Hyper is a third theme, and `light-dark()` has two values only. The hyper
stylesheet sets each colour token again, on `:root[data-theme="hyper"]`, and
sets `color-scheme: dark`. See [Hyper is opt-in](../hyper/).
