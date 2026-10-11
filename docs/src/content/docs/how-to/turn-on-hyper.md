---
title: Turn on the hyper theme
description: Offer the hyper theme in a product.
sidebar:
  order: 2
---

Hyper is off in each product until the product turns it on. Hyper needs Aurora
0.5 or later.

## Procedure

1. Load the hyper stylesheet after the Aurora stylesheet. Use the way that
   matches how you load `aurora.css`:

   - With `<AuroraStyles />`, give it `hyper=true`:

     ```rust
     view! { <AuroraStyles hyper=true /> }
     ```

   - With a linked stylesheet, write `hyper.css` too, and link it after
     `aurora.css`. The `aurora-css` binary writes it with `--hyper`
     (`aurora-css style --hyper`). A helper crate calls
     `aurora_leptos::write_hyper_css(dir)`.

     ```html
     <link data-trunk rel="css" href="style/aurora.css" />
     <link data-trunk rel="css" href="style/hyper.css" />
     ```

   - Or use the `aurora_leptos::HYPER_CSS` constant.

2. Give the theme toggle `hyper=true`:

   ```rust
   view! { <ThemeToggle hyper=true /> }
   ```

   The toggle then shows Light, Dark, Hyper and System. Without a toggle, call
   `set_theme(Theme::Hyper)`.

3. Make sure that the `<head>` has the `THEME_INIT_SCRIPT` of 0.5. The script of
   0.4 does not know `"hyper"`, so a reload shows the system theme. Copy the
   script from [Themes](../../reference/themes/#theme_init_script).

4. If your code matches on `Theme`, add a `Theme::Hyper` arm or a `_` arm.

## Check it

1. Run the app.
2. Click **Hyper** in the toggle. The page turns neon on a dark base, and
   `<html>` has `data-theme="hyper"`.
3. Reload the page. It stays in hyper.

If the page shows the light or the dark theme after you choose Hyper, the
hyper stylesheet is not on the page. With no hyper stylesheet,
`current_theme()` gives `System`, and the page follows the operating system.

## Turn off the glow

You do not need to. The glow goes away when the operating system asks for
more contrast (`prefers-contrast: more`).
