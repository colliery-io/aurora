---
title: Run the gallery
description: Build and open leptos-gallery, the app that shows each Aurora component.
sidebar:
  order: 6
---

`leptos-gallery` is an app in the Aurora repository. It shows each component,
each widget and the colour tokens, with a theme toggle that has hyper. It is
not published: run it on your computer.

## Before you start

You need Rust with the `wasm32-unknown-unknown` target, and
[Trunk](https://trunkrs.dev):

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

## Procedure

1. Clone the repository:

   ```sh
   git clone https://github.com/colliery-io/aurora
   ```

2. Go to the gallery:

   ```sh
   cd aurora/leptos-gallery
   ```

3. Serve it:

   ```sh
   trunk serve --open
   ```

   Before each build, a hook writes `style/aurora.css` and `style/hyper.css`
   from the crate. The browser then opens the gallery.

4. Use the left navigation to go to a component. Use the toggle to change
   the theme.

To build the files only, use `trunk build`. The files go to
`leptos-gallery/dist/`.
