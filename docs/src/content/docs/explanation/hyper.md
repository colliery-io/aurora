---
title: Hyper is opt-in
description: Why the hyper theme is a separate stylesheet that a product must ask for.
sidebar:
  order: 4
---

Hyper is a third theme: neon hues on a dark base, with a soft glow on
accents. It is not in the default stylesheet. A product gets hyper only when
it loads `HYPER_CSS` and offers it in the toggle.

## Why opt-in

- **A product chooses its look.** Light and dark are what a reader expects of
  each product. Hyper is a strong look. A product team decides if it fits.
- **No change for products that do not ask.** Aurora 0.5 added hyper. A
  product that updates to 0.5 and does nothing gets the same stylesheet and
  the same Light / Dark / System toggle as before.
- **No cost for products that do not ask.** The hyper stylesheet is not in
  `AURORA_CSS`, so a product without hyper does not ship it.

## How it stays safe

- **The hues keep their roles.** Ice is cyan, ok is acid green, bad is hot
  red. A status means the same thing in each theme.
- **The same contrast rules.** `tests/contrast.rs` checks hyper with the same
  AA minimums as light and dark. Another test checks that hyper sets each
  token that has a light and a dark value, so no light value shows through.
- **No glow when the reader asks for more contrast.** Each glow is inside
  `@media not (prefers-contrast: more)`.
- **It fails to the system theme.** With no hyper stylesheet on the page,
  `data-theme="hyper"` matches no rule. The page follows the operating
  system, and `current_theme()` gives `System`.

## How it works

`hyper.css` sets each colour token again on `:root[data-theme="hyper"]`, and
sets `color-scheme: dark`. The tints (`--x-bg`) and the gradients are made from
the hues, so they follow. The file also sets `--aurora-hyper-loaded: 1` on
`:root`; `hyper_available()` reads it to know if hyper can show.
