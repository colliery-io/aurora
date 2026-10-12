---
title: Draw a Hlin front end with Aurora
description: Use the hlin feature to make Aurora the design pack of a Hlin front end.
sidebar:
  order: 3
---

[Hlin](https://github.com/colliery-io/hlin) is a shell that shows the panels
of many platforms in one UI. It draws each panel through a design pack. The
`hlin` feature of Aurora gives `AuroraPack`, a design pack made from Aurora's
own components and tokens.

## Procedure

1. Turn on the `hlin` feature, and add `hlin-ui` and Leptos:

   ```toml
   [dependencies]
   colliery-io-aurora = { version = "0.6", features = ["hlin"] }
   hlin-ui = "0.1"
   leptos = { version = "0.8", features = ["csr"] }
   ```

2. Mount the Hlin app with `AuroraPack`:

   ```rust
   use aurora_leptos::AuroraPack;
   use hlin_ui::app::App;
   use leptos::prelude::*;

   fn main() {
       leptos::mount::mount_to_body(|| view! { <App pack=AuroraPack /> });
   }
   ```

That is the whole front end. The pack gives Hlin its stylesheet,
`aurora_leptos::HLIN_CSS`: the Aurora fonts, tokens, component and graph styles, and
the few styles that the pack adds for Hlin.

## What the feature changes

- With the feature off, Aurora does not know Hlin. It builds no Hlin
  dependency, and the default build does not change.
- With the feature on, Aurora adds `AuroraPack` and `HLIN_CSS`, and depends on
  `hlin-view` and `serde_json`. The feature turns on `components` too.

`AuroraPack` implements Hlin's `DesignPack` trait. The trait has no default
methods, so a pack that forgets a kind of view does not compile.
