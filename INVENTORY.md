# aurora-leptos — component inventory

What ships in the pack, grouped as it was in the React design system. Usage
counts are from the consuming app (`cloacina/ui/src`) and indicate how load-bearing
each piece is — useful provenance, not a gate. Everything below is **implemented**.

`MantineProvider` → `AuroraStyles`; `Badge` → `Pill`.

## Layer 1 — primitives (from `@mantine/core`)
There is no Mantine in Rust/WASM, so each is a thin Leptos component over a shared
CSS class (`components.css`) styled from the Aurora tokens.

| Component | Uses | Notes |
|---|---:|---|
| `Box` | 28 | Block / style carrier → `.cl-box`. |
| `Group` | 26 | Horizontal flex; `justify`/`gap`/`wrap` props. |
| `Button` | 19 | Variants filled/light/default/subtle; sizes xs–md; `bad`. |
| `Text` | 15 | size + `dimmed`/`bright`/`bold`/`mono`. |
| `Stack` | 12 | Vertical flex; `gap`/`center`. |
| `Tooltip` | 9 | Pure-CSS hover; multiline + arrow. |
| `Modal` | 9 | Dialog on a bool signal: sizes sm–xl, footer, focus trap, Escape. |
| `TextInput` | 8 | label/placeholder/value/error. |
| `Table` | 6 | `.cl-table` (+ `mono`). |
| `Alert` | 5 | Tinted callout; backs `ErrorState`. |
| `Anchor` | 5 | Accent link. |
| `Switch` | 4 | Controlled toggle. |
| `NumberInput` | 3 | Numeric field + steppers. |
| `Select` | 3 | Styled native `<select>`. |
| `PasswordInput` | 2 | Input + reveal toggle. |
| `Textarea` | 2 | Multi-line field. |
| `SegmentedControl` | 2 | Bound to a string signal. |
| `SimpleGrid` | 2 | Equal-width grid. |
| `AppShell` | 1 | Page scaffold: header, brand, sidebar (drawer below 768px), main. |
| `Menu` | 1 | Dropdown + `MenuItem` (context-wired). |
| `Code` | 1 | Inline monospace chip. |
| `Loader` | 1 | CSS spinner. |
| `ActionIcon` | 1 | Icon-only button. |
| `CopyButton` | 1 | Clipboard + transient confirmation. |
| `Divider` | 1 | Hairline rule. |
| `List` | 1 | `ul` + `ListItem`. |
| `Grid` | 1 | 12-col `Grid` + `GridCol`. |

## Layer 2 — Aurora components + widgets
Generic Aurora pieces, plus the higher-level data-display **widgets**. The
widgets take state labels/colors/tooltips as data — apps supply their own vocab.

| Component / export | Notes |
|---|---|
| `MONO` / `tokens::token` | Mono helper (`.cl-mono`) + palette as CSS variables (`var(--ice)`). |
| `Pill` · `StatusBadge` · `Dot` | Status text token (`--x-fg`) on its fill token (`--x-bg`); status dot. |
| `ThemeToggle` | Light / Dark / System choice, stored in the browser. |
| `Panel` · `PageHeader` · `Chip` | Card surface; page title (back link, meta, actions); filter chip. |
| `SideNav` · `SideNavGroup` · `SideNavLink` | Sidebar nav: groups, active link (`aria-current`), counts, footer. |
| `Drawer` · `ConfirmDialog` | Slide-over panel; confirmation with impacts, type-to-confirm, busy. |
| `ToastStack` · `Toaster` | Toasts from any part of the app (`use_toaster()`). |
| `Tabs` · `TabPanel` · `TabItem` | Underline tab list: signal or route tabs, arrow keys. |
| `Card` | Panel that is a link or a button, with hover and focus states. |
| `Loading` · `Empty` · `ErrorState` | Async-view states (error renders by classified kind). |
| `Meter` | Freshness/progress bar. |
| `Banner` · `StaleInputsBanner` | Tinted callout; stale-inputs convenience. |
| `StateCounts` | Count-per-state badges. |
| `HealthPill` | State pill + tooltip (label/color/tip as data). |
| `BuildStatusBadge` | CI build state → pill. |
| `NodeReadiness` | Trigger description + per-input freshness + summary. |
| `InputTable` | Per-input state/last-event/rate/freshness/action (`Input` model). |

### Pure logic (`tokens.rs`)
Semantic palette (`token::*`), `status_color`, `pill_bg` / `fill_for` /
`fg_for` / `tint`, and `ApiError` classification. `theme.rs` holds the `Theme`
type and `THEME_INIT_SCRIPT`. Framework-agnostic Rust — the seam where typed API models plug in.

## Built downstream (not shipped by the pack)
App branding (e.g. a logo mark) and app-specific state vocab/colors are supplied
by the consuming app as data. Heavier app surfaces — notably DAG/graph + node
views (`@dagrejs/dagre` in React; note `rust-sugiyama`/`layout-rs` for a Rust
port) — are built on these primitives downstream.
