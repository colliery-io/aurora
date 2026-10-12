---
title: Load the stylesheet as a file
description: Link aurora.css in the head, so the first paint is styled.
sidebar:
  order: 1
---

`<AuroraStyles />` puts the stylesheet on the page when the app starts, so
the page can show unstyled content for a moment. A `<link>` in the `<head>`
loads the stylesheet before the first paint. Aurora ships the CSS inside the
crate; `aurora_leptos::write_css(dir)` writes it to `dir/aurora.css`.

## With Trunk

Trunk checks its assets before it builds the crate, so a `build.rs` is too
late. Write the file in a `pre_build` hook. Cargo cannot run a binary of a
dependency, so the hook runs a small helper crate of your own.

1. Make the helper crate `tools/aurora-css/Cargo.toml`:

   ```toml
   [package]
   name = "aurora-css"
   version = "0.0.0"
   edition = "2021"
   publish = false

   [dependencies]
   colliery-io-aurora = { version = "0.6", default-features = false }
   ```

   With `default-features = false` the helper does not build Leptos.

2. Write `tools/aurora-css/src/main.rs`:

   ```rust
   fn main() {
       let dir = std::env::args().nth(1).unwrap_or_else(|| "style".into());
       let path = aurora_leptos::write_css(std::path::Path::new(&dir)).expect("write aurora.css");
       eprintln!("aurora-css: wrote {}", path.display());
   }
   ```

3. Add `"tools/aurora-css"` to `members` in the `[workspace]` of your root
   `Cargo.toml`. The helper then uses the same `Cargo.lock`, and the same
   Aurora version, as the app.

4. Add the hook to `Trunk.toml`:

   ```toml
   [[hooks]]
   stage = "pre_build"
   command = "cargo"
   command_arguments = [
     "run", "-q", "-p", "aurora-css",
     "--target-dir", "target/_aurora_css",
     "--", "style",
   ]
   ```

   The separate target folder stops the hook from rebuilding the app.

5. In `index.html`, link the file after `THEME_INIT_SCRIPT`:

   ```html
   <script>/* THEME_INIT_SCRIPT */</script>
   <link data-trunk rel="css" href="style/aurora.css" />
   ```

6. Remove `<AuroraStyles />` from the app.
7. Add `style/aurora.css` to `.gitignore`. The hook makes it.

If your product is not a Cargo workspace, give the helper crate its own
`[workspace]` table, and run it with `--manifest-path tools/aurora-css/Cargo.toml`.
Pin the same Aurora version as the app.

### Or install the binary

The crate has a binary, `aurora-css`, behind the `bin` feature. Install it on
each machine and in CI, at the version of the app:

```sh
cargo install colliery-io-aurora --locked --version 0.6.1 --no-default-features --features bin
```

Then the hook is:

```toml
[[hooks]]
stage = "pre_build"
command = "aurora-css"
command_arguments = ["style"]
```

`aurora-css <dir>` writes `<dir>/aurora.css`. With `--hyper` it also writes
`<dir>/hyper.css`.

## With cargo-leptos

cargo-leptos builds the crate before it processes the styles, so a
`build.rs` works.

1. Add Aurora to `[build-dependencies]`, with
   `default-features = false`.
2. In `build.rs`, call `aurora_leptos::write_css(...)` with a folder.
3. Point `style-file` in `[package.metadata.leptos]` at the `aurora.css` that
   it writes.

## Fonts

The stylesheet loads IBM Plex from Google Fonts. To ship with no network, host
the fonts yourself.
