---
title: Use a token in your product
description: Colour your own CSS and inline styles with Aurora tokens, so they follow the theme.
sidebar:
  order: 4
---

Use a token for each colour in your product. A token follows the theme; a
raw colour does not. The tokens are in the [token reference](../../reference/tokens/).

## In your CSS

Use `var(--name)`. Load your stylesheet after the Aurora stylesheet.

```css
.app-sidebar-count {
  background: var(--ice-bg);
  color: var(--ice-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  font-family: var(--font-mono);
}
```

## In Rust

1. Import the tokens:

   ```rust
   use aurora_leptos::tokens::{tint, token};
   ```

2. Use a `token::` constant in an inline style. Each constant is a
   `var(--x)` string:

   ```rust
   view! { <span style=format!("color:{};", token::BAD)>"3 failed"</span> }
   ```

3. For a lighter tint of a hue, use `tint`. Do not add hex alpha to a token:
   `format!("{}1c", token::ICE)` is not a colour.

   ```rust
   let fill = tint(token::ICE, 20); // color-mix(in srgb, var(--ice) 20%, transparent)
   ```

4. For a tinted fill with text on it, use the pair:
   `token::OK_BG` behind `token::OK_FG`. Or give the hue to `Pill`, which
   uses the pair for you.

A token with no `token::` constant is still a CSS variable. In Rust, write it
as a string: `"var(--border)"`.

## Change a token in one product

Do this only when the product must differ from Aurora. The contrast test of
Aurora does not check your values.

1. In your stylesheet, set the token on `html:root`. Give a light and a dark
   value:

   ```css
   html:root {
     --ice: light-dark(#2457b3, #8cb6ff);
   }
   ```

   `html:root` has more weight than Aurora's `:root`, so the order of the
   stylesheets does not matter.

2. Check the contrast of the new value, in the light and the dark theme, on
   each surface where it shows. Text needs 4.5:1.
3. If the token has a `-fg` text token, change it too.
4. If the product offers hyper, set the hyper value as well, on
   `html:root[data-theme="hyper"]`. Hyper sets each colour token on
   `:root[data-theme="hyper"]`, which has more weight than `html:root`, so the
   value of step 1 does not show in hyper.

A tint token (`--x-bg`) is made from its hue. When you change `--ice`,
`--ice-bg` follows.
