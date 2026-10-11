---
title: Rules and checks
description: The rules that keep Aurora's colours right, and the tests that check them.
sidebar:
  order: 6
---

The tests are in `aurora-leptos/tests/`. Run them all with:

```sh
cargo test -p colliery-io-aurora
```

## No raw colours

Rule: a colour in the crate or the gallery is a token. Write `var(--x)` in CSS
and `token::X` in Rust.

`tests/no_raw_colours.rs` checks it.

| | |
|---|---|
| Looks in | The `.css` and `.rs` files of `aurora-leptos/style`, `src` and `tests`; `leptos-gallery/src`; `leptos-gallery/gallery.css` |
| Finds | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(`, `rgba(`, `hsl(`, `hsla(` |
| Skips | `style/tokens.css` and `style/hyper.css` (they define the values), and the two colour tests |
| Exceptions | A line in the `ALLOW` list of the test. Each entry must give a reason. The list is empty. |

A named colour (`red`) is not found by the test, but the rule is the same.

## Contrast

Rule: each text and fill pair that the components use meets WCAG AA, in the
light, the dark and the hyper theme.

`tests/contrast.rs` checks it. It reads the values from `style/tokens.css`
and `style/hyper.css`. A token that hyper does not set takes its dark value.

| Pair | Minimum |
|---|---|
| Each text token on each surface | 4.5:1 |
| Each `--x-fg` on its `--x-bg` fill, over the page and the panels | 4.5:1 |
| Large text (`--fainter`) | 3:1 |
| The edge of a control (`--border-control`), graph edges (`--edge`), the focus ring (`--ice`) | 3:1 |
| A hue as a sparkline, a segment or a meter fill | 3:1 |

To see the full table:

```sh
cargo test -p colliery-io-aurora --test contrast -- --nocapture
```

## Hyper

| Test | Checks |
|---|---|
| `tests/contrast.rs`, `hyper_sets_every_themed_token` | `hyper.css` sets each token that has a `light-dark()` value. |
| `tests/hyper.rs` | Hyper is not in `AURORA_CSS`; it sets `color-scheme: dark` and the marker that `hyper_available()` reads; each glow is off with `prefers-contrast: more`; `write_hyper_css` writes the file. |

## The init script

`tests/theme_init.rs` checks that the gallery's `index.html` has
`THEME_INIT_SCRIPT`, exactly, in its `<head>`, before the stylesheet.
