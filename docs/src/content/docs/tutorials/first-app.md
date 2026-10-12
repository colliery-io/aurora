---
title: Add Aurora to a Leptos app
description: Make a small Leptos app with the Aurora stylesheet, a theme toggle and a first panel.
sidebar:
  order: 1
---

In this tutorial you make a small Leptos app that runs in the browser. It has
the Aurora stylesheet, a theme toggle, and a panel with status badges and a
button. It takes about ten minutes, most of it the first build.

## Before you start

You need:

- Rust, stable, with the WebAssembly target:

  ```sh
  rustup target add wasm32-unknown-unknown
  ```

- [Trunk](https://trunkrs.dev), which builds and serves the app:

  ```sh
  cargo install trunk --locked
  ```

## 1. Make the crate

```sh
cargo new aurora-hello
cd aurora-hello
```

## 2. Add Aurora and Leptos

Open `Cargo.toml`. Set `[dependencies]` to this:

```toml
[dependencies]
colliery-io-aurora = "0.6"
leptos = { version = "0.8", features = ["csr"] }
```

The crate is `colliery-io-aurora`. In Rust code you import it as
`aurora_leptos`. Leptos must be 0.8. The `csr` feature makes an app that runs
in the browser.

## 3. Write the page

Make a file `index.html` next to `Cargo.toml`:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Aurora hello</title>
    <meta name="color-scheme" content="light dark" />
    <script>(function(){try{var t=localStorage.getItem("aurora-theme");if(t==="light"||t==="dark"||t==="hyper"){var d=document.documentElement;d.setAttribute("data-theme",t);if(t!=="hyper")d.style.colorScheme=t;}}catch(e){}})();</script>
  </head>
  <body></body>
</html>
```

The `<script>` is `aurora_leptos::THEME_INIT_SCRIPT`, copied as it is. It
reads the stored theme and sets it on `<html>` before the first paint, so the
page does not flash the wrong theme while the app loads. Keep it in the
`<head>`.

## 4. Write the app

Replace `src/main.rs` with this:

```rust
use aurora_leptos::components::*;
use aurora_leptos::theme::{provide_theme, ThemeToggle};
use aurora_leptos::tokens::token;
use aurora_leptos::AuroraStyles;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    provide_theme();
    let runs = RwSignal::new(0);

    view! {
        <AuroraStyles />
        <Stack>
            <Group justify="between">
                <Text size="lg" bold=true>"Hello, Aurora"</Text>
                <ThemeToggle />
            </Group>
            <Panel title="Jobs" caption="today">
                <Stack gap="sm">
                    <Group>
                        <StatusBadge status="running" />
                        <StatusBadge status="failed" />
                        <Pill color=token::OK>"healthy"</Pill>
                    </Group>
                    <Group>
                        <Button on_click=Callback::new(move |_| runs.update(|n| *n += 1))>
                            "Run"
                        </Button>
                        <Text dimmed=true>{move || format!("Runs: {}", runs.get())}</Text>
                    </Group>
                </Stack>
            </Panel>
        </Stack>
    }
}
```

What each part does:

- `<AuroraStyles />` puts the Aurora stylesheet on the page.
- `provide_theme()` makes the theme state. Call it once, at the root.
- `<ThemeToggle />` shows Light, Dark and System. The choice is stored in the
  browser under the key `aurora-theme`.
- `<StatusBadge status="running" />` shows a status. Aurora picks the hue from
  the text: running is ice, failed is bad.
- `<Pill color=token::OK>` shows a label in a hue that you choose.
  `token::OK` is the string `"var(--ok)"`, so the pill follows the theme.

## 5. Run the app

```sh
trunk serve --open
```

The first build takes a minute or two. Then the browser opens the app. You
see a title, the theme toggle, and a panel named "Jobs" with two status
badges, a "healthy" pill and a "Run" button.

1. Click **Run**. The count goes up.
2. Click **Dark**. The page changes to the dark theme.
3. Reload the page. It stays dark, with no flash of the light theme.
4. Click **System**. The page follows the operating system again.

## What you made

You have a Leptos app that draws with Aurora, in a light and a dark theme,
and that remembers the reader's choice.

## Next

- [Load the stylesheet as a file](../../how-to/link-the-stylesheet/):
  `<AuroraStyles />` can show unstyled content for a moment while the app
  loads. A linked stylesheet does not.
- [Turn on the hyper theme](../../how-to/turn-on-hyper/).
- [Components](../../reference/components/): each component and its props.
