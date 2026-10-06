# Changelog

## 0.4.2

A patch release with the repairs of AURORA-T-0008: the defects that the
Cloacina migration found, and that 0.4.1 kept. Nothing is removed or renamed.
The numbers are the items of AURORA-T-0008. `tests/aurora_t_0008.rs` has a
test for each item.

- **1.** `Button bad=true` colours the `subtle` and `default` variants too: the
  danger text colour, and a danger tint on hover. (0.4.1 coloured only
  `filled` and `light`.)
- **2.** The placeholder of a password input has normal letter-spacing. The
  wide spacing is for the bullets only.
- **3.** `StatTile` has a `text` prop for a value that is text (a name,
  "just now"): a smaller size (`--fs-xl`), on one line, with an ellipsis. The
  full value is the `title`. The gallery has two examples.

Cloacina can now remove the "Workarounds for Aurora 0.4" rules from its
`ui/style/app.css` (`.cl-btn--bad.cl-btn--subtle`, `.cl-btn--bad.cl-btn--default`,
the password placeholder rule, `.app-stat-text`).

## 0.4.1

A patch release with the repairs of AURORA-T-0007: the defects that the
Weir, Brokkr and Kairos migrations to 0.4.0 found. Nothing is removed or
renamed, and each 0.4.0 call shape still compiles. The numbers are the items
of AURORA-T-0007. `tests/aurora_t_0007.rs` has a test for each item.

### Inputs

- **1.** `mono=true` on `TextInput` and `Textarea` now sets the mono font. A rule
  `.cl-input.cl-mono` comes after `.cl-input` in `components.css`.
- **2.** `Select` has an `aria_label` prop, on the `<select>` itself. (`attr:` still
  goes on the wrapper `<div>`.) `TextInput` and `Textarea` have it too.
- **8.** `Select` with a value that no option has, and no placeholder, shows the
  first option, as a native select does. The `value` signal is not changed.
  `select_shown_value` is the rule.
- **12.** `error` of `TextInput`, `Select`, `Textarea`, `NumberInput` and
  `PasswordInput` is a `Signal<String>`: a `String`, a `&str`, a signal or a
  closure. The message, the red border, `aria-invalid` and
  `aria-describedby` follow it. The five inputs have an `id` prop, so a label
  outside the component can point at the control.

### Layout and states

- **3.** `SimpleGrid` collapses: at most 2 columns below 768px, one below 480px.
  The count of columns is the CSS variable `--cl-cols`, not an inline
  `grid-template-columns`. `fixed=true` keeps the count at every width.
- **9.** `Meter indeterminate=true`: a bar for work of unknown length. It is a
  `role="progressbar"` with no value, named by `label` (default "Loading").
  With reduced motion it is a still bar. `value` is now optional.
- **10.** `Chip` is a `type="button"` with `aria-pressed`.
- **11.** `Group` has an `align` prop: `"start"`, `"center"` (default), `"end"`
  (the bottom: a labelled field and a button line up), `"baseline"`,
  `"stretch"`.
- **20.** `Empty` has a next step: `hint` (a line under the message), `href` and
  `link` (a link, "Read how" by default), and children (an action).

### Data components

- **6.** `FeedRow` with no `at` and no `time` has no time column, and no 64px gap.
  `keep_time_slot=true` keeps the empty column, to line up with rows that
  have a time.
- **7.** A long `sub` (or delta) line of `StatTile` wraps inside the tile, even a
  word with no break (`overflow-wrap: anywhere`, `min-width: 0`).
- **14.** `Pagination` has `show_range` (default true) and `range_label`, a
  `Callback<PageRange, String>` that makes the range text.

### Frame and dialogs

- **4.** The `title` of `Modal`, `Drawer` and `ConfirmDialog` is a
  `Signal<String>`: a string, a signal or a closure
  (`title=move || format!("Delete {}?", name.get())`).
- **13.** With an `AppShell` that fills the page, `<html>` has `scroll-padding-top`
  (the header height and a little air), so an anchor or `scrollIntoView`
  does not put its target under the sticky header.
- **18.** `ConfirmDialog` has a `notice` slot: content between the message and the
  impacts list, such as a warning. The children stay after the list.

### Motion

- **5.** `.cl-pulse` stops when the OS asks for less motion, wherever it is used
  (the guard was only inside `.cl-dag`).

### Graph

- **15.** A `Dag` edge has `data-style` (the style name), `data-from` and `data-to`.
  The marker ids are stable: `{id}-arrow-{style}` (`dag_marker_id`), with a
  new `id` prop on `Dag` (default a unique `cl-dag-N`, also the id of the
  `<svg>`). Before, the marker ids held the index of the style.
- **16.** In `Dag`, the second click of a double click no longer selects again.
  With `defer_select=true`, a click waits 250ms (`DOUBLE_CLICK_MS`) before
  `on_select`, and a double click runs only `on_open`. (`click_intent` is
  the rule; the timing itself is not tested.)
- **17.** A node and its "+N" badge are in one `<g class="cl-dag__item"
  data-node="…">`. The hover covers both, the badge dims with its node, and
  `[data-node=id] .cl-dag__more` finds it. The badge is not inside the
  node's `role="button"`, because a button in a button has no accessible
  name of its own. `.cl-dag__nodes` now holds the items, not the nodes.

### Docs

- **19.** The README gives a trunk path with no global install: a small helper crate
  in the product's workspace that calls `write_css` (leptos-free, it shares
  the `Cargo.lock`). The install line of the `aurora-css` bin pins the
  version. `cargo run -p colliery-io-aurora` works only in this repository,
  and the docs now say so.

### Tests

- The components render to HTML in native tests: `leptos` with `ssr` and
  `any_spawner` are dev-dependencies. A dialog no longer reads the focused
  element outside a browser.
- Not tested in Rust: the cascade at run time (items 1, 3, 7, 13 are checked
  in the stylesheet) and the timing of a double click (item 16).


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
