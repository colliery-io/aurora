# Changelog

## 0.4.0

Aurora Dark is now **Aurora**. It has a light theme and a dark theme. The
crate name `colliery-io-aurora` and the repository name do not change.

### Changes that can break a product

Do these when you move to 0.4.0.

1. **The removed widgets.** `NodeReadiness`, `InputTable`, `StaleInputsBanner`,
   `StateCounts`, the model `widgets::Input`, `STALE_MS`, `format_ago`,
   `is_stale` and `freshness_pct` are gone, with their CSS (`cl-state-count`,
   `cl-readiness*`, `cl-input-table*`). No Colliery product used them. Use
   `RelativeTime` or `format_relative` for "time ago", and `SegmentedBar` or
   `Pill` for counts per state.
2. **`token::*` are CSS variables.** `token::ICE` is now `"var(--ice)"`, and
   not `"#7fb2ff"`. Code that adds text to a token no longer makes a colour.
   For example, `format!("{}7a", token::ICE)` makes the invalid
   `var(--ice)7a`. Use `tint(token::ICE, 48)`. Code that compares two tokens
   still works.
3. **The page follows the theme of the operating system.** A colour that a
   product writes itself (a hex value in its CSS, an `rgba(255,255,255,…)`
   overlay) looks wrong in the light theme. Move it to a token. Until then,
   put `data-theme="dark"` on `<html>` to keep the dark theme.
4. **`AppShell` fills the page.** It has a sticky header and sidebar, and
   `<header>` and `<main>` elements. An app that has its own `<main>` inside
   `AppShell` now has two. Below 768 px the sidebar is a drawer behind a menu
   button.
5. **`Modal` closes with Escape** and moves the focus. Use `locked` to stop a
   close while work runs.
6. **`Button` markup.** The label is in `<span class="cl-btn__label">`, and the
   button has `aria-busy`. Check CSS that targets a direct child of `.cl-btn`.
   `variant="primary"` now draws as `filled`.
7. **`CopyButton`** says "Copied", and not "Copied!". It shows nothing when the
   browser has no Clipboard API, for example on a page served over plain HTTP.
8. **`PasswordInput`.** The title of the reveal button is "Show password" or
   "Hide password", and not "Toggle visibility".
9. **`Graph`** is a wrapper over the new `Dag`. Its API does not change, but
   the layout and the look of the nodes do.
10. **Four dark colours are lighter**, so that text and borders meet WCAG AA:
    `--faint`, `--fainter`, `--border-control` and `--edge`.

### Themes and tokens

- Each colour is `light-dark(<light>, <dark>)` in `style/tokens.css`, which is
  the only file with raw colours. A test refuses a raw colour anywhere else.
- With no choice, the theme follows `prefers-color-scheme`. `data-theme="light"`
  or `data-theme="dark"` on `<html>` forces a theme. It needs a browser of 2024
  or later.
- `ThemeToggle` offers Light, Dark and System. The browser keeps the choice
  under `aurora-theme`. The API is in `aurora_leptos::theme`.
- Put `THEME_INIT_SCRIPT` in the `<head>`, before the stylesheet, so that the
  first paint has the correct theme.
- Each status hue (ice, teal, violet, gold, ok, bad, skip, muted) has `--x`,
  `--x-fg` (text on the fill) and `--x-bg` (the fill). The Rust constants are
  `X`, `X_FG` and `X_BG`. `pill_bg` returns the fill.
- New tokens for fields, hover, tooltips, the scrim, shadows and scroll bars.
- A test checks WCAG AA for each pair of text and background in the two themes.

### New components

- **Frame and dialogs:** `AppShell` v2, `SideNav`, `SideNavGroup`,
  `SideNavLink`, `PageHeader` v2 (back link, metadata, actions), `Modal` v2
  (sizes sm to xl, footer, focus trap), `Drawer`, `ConfirmDialog`, toasts
  (`provide_toaster`, `use_toaster`, `ToastStack`), `Tabs` and `TabPanel`,
  `Card`.
- **Graph:** `Dag` does the layout and the interaction: ranks or fixed layers,
  lanes, fewer crossings, selection, hover, keyboard, and a legend. The layout
  is the pure module `graph_layout`.
- **Data:** `StatTile`, `Sparkline`, `SegmentedBar`, `DetailList` and
  `KeyValue`, `SectionLabel`, `CodeBlock`, `LogView`, `FeedList` and
  `FeedRow`, `Table` v2 (`TableRow`, `SortHeader`, `TableEmpty`),
  `Pagination`, `RelativeTime`, `LiveIndicator`, `CopyButton` v2,
  `SecretReveal`, `CenterScreen` and `AuthCard`, `Menu` v2, and 15 icons.

### Inputs and buttons

- `disabled` accepts a bool, a signal or a closure.
- `Button` has `loading`, `button_type`, `href` and `stop_propagation`.
- The fields have `on_input`, `on_change`, `name`, `autocomplete`, `required`
  and `error`. `Select` has `option_pairs` and `placeholder`.
- `attr:` puts each other attribute on the root element.
- `Tooltip` has 4 positions and a delay, and it shows on keyboard focus.

### Accessibility

- `Modal` has `role="dialog"`, `aria-modal` and a label. `PageHeader` has an
  `<h1>`. Each field label points to its control.
