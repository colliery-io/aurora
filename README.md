# Aurora — Leptos design pack (`colliery-io-aurora`)

[![crates.io](https://img.shields.io/crates/v/colliery-io-aurora.svg)](https://crates.io/crates/colliery-io-aurora)
[![docs.rs](https://docs.rs/colliery-io-aurora/badge.svg)](https://docs.rs/colliery-io-aurora)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

Colliery's **design system for [Leptos](https://leptos.dev)**, with a light
and a dark theme — the Aurora identity (tokens, components, data-display
widgets, stylesheet) as a reusable Rust/WASM crate. Aurora was formerly called
"Aurora Dark"; the crate name (`colliery-io-aurora`) and the repository name
(`aurora-dark`) stay. It's the core that control-plane apps (cloacina
included) are built from; app-specific vocab, colors, and branding are supplied
as data, not shipped.

## When to use this
Reach for `colliery-io-aurora` when you're building a **control-plane / dashboard
Leptos UI** for a Colliery project and want the chrome handled: design tokens, the
full primitive set (layout, inputs, overlays, tables), async-state components
(`Loading`/`Empty`/`ErrorState`), data-display widgets (status pills, freshness
meters, readiness panels), and graph/DAG drawing — so you build screens, not a
component library. You supply the meaning (state→color/label maps, copy, branding)
as data. **Not** the right fit for non-Leptos stacks or a general public-facing
marketing site. See
[`aurora-leptos/PATTERNS.md`](./aurora-leptos/PATTERNS.md) for which component to
reach for, task by task.

## Layout
```
aurora-leptos/       # ★ the design-system crate (published as colliery-io-aurora)
  src/
    lib.rs           #   public API: components, tokens, AURORA_CSS / <AuroraStyles/>
    components.rs    #   core components (primitives)
    tokens.rs        #   semantic tokens (CSS variables) + error classification
    theme.rs         #   light / dark / system: ThemeToggle, set_theme, init script
    widgets.rs       #   generic data-display widgets (Meter, Banner, …)
    hlin.rs          #   `hlin` feature: Aurora as a Hlin design pack
    hlin.css         #     the chrome that module needs, embedded in it
  style/             #   framework-agnostic stylesheet, shipped with the crate
    tokens.css       #     Aurora tokens: light + dark colours, spacing, radii, type scale
    components.css   #     every component's static chrome
    fonts.css        #     IBM Plex @font-face
  PATTERNS.md        #   usage guide — when to reach for each component
leptos-gallery/      # example app rendering every component/widget
INVENTORY.md         # component inventory
```

**New here?** Read `aurora-leptos/PATTERNS.md` — a pick-by-intent usage guide for
people and AI agents.

## Consume it

Published on crates.io as **`colliery-io-aurora`** (org-prefixed so we don't claim
generic names in the flat crates.io namespace); the library is still imported as
`aurora_leptos`.

```toml
[dependencies]
colliery-io-aurora = "0.1"
leptos = { version = "0.8", features = ["csr"] }   # match 0.8.x; binary picks the renderer
```

Or as a git dependency (Cargo finds the crate in this repo's subdir):

```toml
[dependencies]
colliery-io-aurora = { git = "https://github.com/colliery-io/aurora-dark", rev = "<commit-sha>" }
```
```rust
use aurora_leptos::{components::*, widgets::*, graph::*, tokens::token};
```

### Styling — pick one
The stylesheet ships inside the crate, so you either inject it at runtime or
materialise it as a file at build time. `write_css`/the `aurora-css` helper are
**leptos-free** (`default-features = false`), so the build step never compiles
leptos for the host.

- **Runtime (simplest, any toolchain).** Render `<AuroraStyles/>` once at the app
  root — it `include_str!`s the CSS into the wasm and injects a `<style>`. Zero
  build config; trade-off is a possible first-paint flash (matters most under SSR).

- **Linked stylesheet, no flash — trunk.** A `<head>` `<link>` is render-blocking
  (no flash), but trunk validates assets *before* building, so a `build.rs` can't
  emit the file in time. Generate it in a **`pre_build` hook** instead:
  ```toml
  # Trunk.toml — install the helper once:
  #   cargo install colliery-io-aurora --no-default-features --features bin
  [[hooks]]
  stage = "pre_build"
  command = "aurora-css"
  command_arguments = ["style"]      # writes style/aurora.css
  ```
  ```html
  <link data-trunk rel="css" href="style/aurora.css" />
  ```
  (In a workspace that *contains* the crate, skip the install and run it via
  `cargo run -p colliery-io-aurora … --bin aurora-css` — see
  `leptos-gallery/Trunk.toml`, which dogfoods exactly this.)

- **Linked stylesheet — cargo-leptos.** It builds the crate before processing
  styles, so `aurora_leptos::write_css(...)` from a `build.rs` works there; point
  `[package.metadata.leptos] style-file` at the output.

Fonts load from Google Fonts at runtime — self-host if you ship fully offline.
Prefer `rev`/`tag` over `branch`. See `aurora-leptos/PATTERNS.md` for which
component to use, and `aurora-leptos/README.md` for the API.

## Light and dark theme

Aurora has a light and a dark theme. By default the page follows the operating
system (`prefers-color-scheme`), and changes when the OS setting changes. The
`data-theme` attribute on `<html>` forces a theme:

| `<html>` | Theme |
|---|---|
| no `data-theme` | follows the OS |
| `data-theme="light"` | light |
| `data-theme="dark"` | dark |

Each colour token holds both values: `--bg: light-dark(<light>, <dark>)` in
`style/tokens.css`. The attribute sets `color-scheme`, and the browser picks the
value. Native controls (scrollbars, select popups) follow too.

To keep a product dark for now, put `data-theme="dark"` on its `<html>`.

**Let the user choose.** Put the toggle in your top bar, and call
`provide_theme()` once at the root:

```rust
use aurora_leptos::theme::{provide_theme, ThemeToggle};

#[component]
fn App() -> impl IntoView {
    provide_theme();
    view! { <header> /* ... */ <ThemeToggle /> </header> }
}
```

`ThemeToggle` shows Light / Dark / System. The choice is stored in
`localStorage` under `aurora-theme`. If storage is not available, the page
follows the OS and shows no error. Without the component, use
`set_theme(Theme::Dark)`, `current_theme()`, or `use_theme()` (a
`ThemeContext` with a `choice` signal and a live `system_dark` signal).

**No flash at load.** Put `aurora_leptos::THEME_INIT_SCRIPT` in an inline
`<script>` in the `<head>`, before the stylesheet. It sets `data-theme` from
storage before the first paint. Copy it from `leptos-gallery/index.html`:

```html
<meta name="color-scheme" content="light dark" />
<script>(function(){try{var t=localStorage.getItem("aurora-theme");if(t==="light"||t==="dark"){var d=document.documentElement;d.setAttribute("data-theme",t);d.style.colorScheme=t;}}catch(e){}})();</script>
<link data-trunk rel="css" href="style/aurora.css" />
```

## Colour tokens

In Rust, `token::*` are CSS variable references (`token::ICE` is
`"var(--ice)"`), so an inline style follows the theme. Do not add hex alpha to
them (`format!("{}1c", token::ICE)` does not work); use `tint(color, pct)`,
`fill_for(color)` or `pill_bg(color)`.

Each status hue has three tokens:

| Token | Rust | Use |
|---|---|---|
| `--x` | `token::X` | the hue: dots, meters, count badges, and text on a page or panel |
| `--x-fg` | `token::X_FG` | text on the `--x-bg` fill (pills, badges) |
| `--x-bg` | `token::X_BG` | the tinted fill behind `--x-fg` |

`x` is `ice`, `teal`, `violet`, `gold`, `ok`, `bad`, `skip` or `muted`. Text on a
solid hue fill uses `--on-status` (`token::ON_STATUS`). `Pill` and `StatusBadge`
use the pair when you give them a hue token.

All text/fill pairs meet WCAG AA in both themes. `tests/contrast.rs` checks
them; `tests/no_raw_colours.rs` refuses a raw colour (hex, `rgb()`, `hsl()`)
anywhere in the crate or the gallery outside `style/tokens.css`.

## Drawing a Hlin surface

[Hlin](https://github.com/colliery-io/hlin) is a composition shell over
independently released platforms: each platform declares what it can show, and
the shell draws every panel through one design system. The `hlin` feature makes
Aurora that design system.

```toml
[dependencies]
colliery-io-aurora = { version = "0.1", features = ["hlin"] }
hlin-ui = "0.0.1"
leptos = { version = "0.8", features = ["csr"] }
```

```rust
use aurora_leptos::AuroraPack;
use hlin_ui::app::App;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(|| view! { <App pack=AuroraPack /> });
}
```

That is a whole Hlin front end. `AuroraPack` implements Hlin's `DesignPack`
trait — ten methods, no defaults, so a pack that forgets a view kind does not
compile — and draws panels out of Aurora's own components, tokens and widgets
rather than a second set that happens to look similar.

The feature is additive and off by default. With it off this crate does not
know Hlin exists, compiles no extra dependency, and nothing about the default
build changes.

`HLIN_CSS` carries the chrome the pack needs on top of `AURORA_CSS`.

## Build & run the gallery
```
cd leptos-gallery && trunk serve --open       # dev
cd leptos-gallery && trunk build --release     # ship
```

## Design principle
The **styling layer is plain CSS** (owned by `aurora-leptos/style/`) and the
**pure logic is plain Rust** (`tokens.rs`). Components are thin Leptos wrappers
over that — and apps supply their own state labels/colors/branding as data, so
the pack stays generic.

## Releasing
Bump `version` in `aurora-leptos/Cargo.toml`, then push a semver tag
(`git tag v0.x.y && git push --tags`) — CI publishes the crate to crates.io
(`.github/workflows/publish.yml`).

## License
[Apache-2.0](./LICENSE)
