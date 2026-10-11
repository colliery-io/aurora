---
title: Cargo features and stylesheets
description: The features of colliery-io-aurora, and the constants and functions that give the CSS.
sidebar:
  order: 5
---

## Cargo features

| Feature | Default | What it turns on |
|---|---|---|
| `components` | yes | The Leptos components, the theme functions and `AuroraStyles`. It depends on `leptos` 0.8, `web-sys`, `js-sys` and `wasm-bindgen-futures`. |
| `hlin` | no | `AuroraPack` and `HLIN_CSS`: Aurora as a [Hlin](https://github.com/colliery-io/hlin) design pack. It depends on `hlin-view` and `serde_json`, and it turns on `components`. |
| `bin` | no | The `aurora-css` binary, which writes the stylesheet to a folder. |

With `default-features = false` the crate builds no Leptos. It still has the
CSS constants, `write_css`, `write_hyper_css`, the tokens, the `Theme` type,
`THEME_INIT_SCRIPT` and the graph layout. Use that in a build script or a
helper crate.

The crate does not choose a Leptos renderer. The app turns on `csr`,
`hydrate` or `ssr` in its own `leptos` dependency.

## Stylesheet constants

Each constant is a `&str` of CSS, in the crate at build time.

| Constant | Holds |
|---|---|
| `AURORA_CSS` | The full stylesheet: fonts, tokens, components and the graph. |
| `TOKENS_CSS` | The tokens only. |
| `COMPONENTS_CSS` | The component styles only. They need the tokens. |
| `GRAPH_CSS` | The graph styles only (`Dag`, `DagLegend`, `Graph`). They need the tokens. |
| `FONTS_CSS` | The IBM Plex `@font-face` rules only. |
| `HYPER_CSS` | The opt-in hyper theme. Not in `AURORA_CSS`. Load it after it. |
| `HLIN_CSS` | With `hlin`: the stylesheet of a Hlin front end. Fonts, tokens, components and the Hlin styles. |

## Stylesheet functions

| Function | What it does |
|---|---|
| `write_css(dir)` | Writes `AURORA_CSS` to `dir/aurora.css`, and makes `dir` if needed. Gives the path. |
| `write_hyper_css(dir)` | Writes `HYPER_CSS` to `dir/hyper.css`. Gives the path. |
| `<AuroraStyles />` | Puts `AURORA_CSS` in an inline `<style>`. `hyper=true` adds `HYPER_CSS`. |

## The aurora-css binary

```sh
aurora-css [dir] [--hyper]
```

| Argument | Default | What it does |
|---|---|---|
| `dir` | `style` | The folder for `aurora.css`. |
| `--hyper` | off | Also writes `hyper.css`. |

Install it with:

```sh
cargo install colliery-io-aurora --locked --no-default-features --features bin
```

See [Load the stylesheet as a file](../../how-to/link-the-stylesheet/).
