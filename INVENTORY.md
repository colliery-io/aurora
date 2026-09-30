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
| `Button` | 19 | Variants filled/light/default/subtle; sizes xs–md; `bad`; reactive `disabled`; `loading`; `href`. |
| `Text` | 15 | size + `dimmed`/`bright`/`bold`/`mono`. |
| `Stack` | 12 | Vertical flex; `gap`/`center`. |
| `Tooltip` | 9 | Pure CSS: hover (after a delay) and keyboard focus, Escape hides it, 4 placements. |
| `Modal` | 9 | Dialog on a bool signal: sizes sm–xl, footer, focus trap, Escape. |
| `TextInput` | 8 | label/placeholder/value/error, `input_type`, reactive `disabled`, `on_input`/`on_change`, `name`/`autocomplete`/`required`. |
| `Table` | 6 | `.cl-table` (+ `mono`, `fixed`, `widths`, `min_width`) + `TableRow` (clickable) · `SortHeader` (`aria-sort`) · `TableEmpty`. |
| `Alert` | 5 | Tinted callout; backs `ErrorState`. |
| `Anchor` | 5 | Accent link. |
| `Switch` | 4 | Controlled toggle. |
| `NumberInput` | 3 | Numeric field + steppers. |
| `Select` | 3 | Styled native `<select>`; `option_pairs` (value, label), `placeholder`. |
| `PasswordInput` | 2 | Input + reveal toggle. |
| `Textarea` | 2 | Multi-line field. |
| `SegmentedControl` | 2 | Bound to a string signal. |
| `SimpleGrid` | 2 | Equal-width grid. |
| `AppShell` | 1 | Page scaffold: header, brand, sidebar (drawer below 768px), main. |
| `Menu` | 1 | Dropdown + `MenuItem` / `MenuLabel` / `MenuDivider`: custom trigger, `align`, `up`, keyboard, Escape and outside click. |
| `Code` | 1 | Inline monospace chip. |
| `Loader` | 1 | CSS spinner. |
| `ActionIcon` | 1 | Icon-only button. |
| `CopyButton` | 1 | Clipboard + confirmation (`aria-live`); `icon`, `link`; not there without the Clipboard API. |
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
| `Meter` | Freshness/progress bar (0–100, a signal). |
| `Banner` | Tinted callout. |
| `HealthPill` | State pill + tooltip (label/color/tip as data). |
| `BuildStatusBadge` | CI build state → pill. |
| `StatTile` · `Sparkline` | KPI tile (label, value, unit, delta or sub line, hue, sparkline slot); tiny line or bar chart in `currentColor`. |
| `SegmentedBar` | Proportions in one bar, with a legend. |
| `DetailList` · `KeyValue` | Label / value rows; mono; dividers; stacked. |
| `SectionLabel` | Mono uppercase heading with a count and an action. |
| `CodeBlock` · `LogView` | Mono block on `--inset`: scroll, max height, copy; log lines with time and level, follow the tail. |
| `FeedList` · `FeedRow` | Activity / run feed: time, dot, subject, actor, status, text, link or button. |
| `Pagination` | Previous / Next, "41–60 of 212", rows per page (offset + limit). |
| `RelativeTime` | "3m ago" in `<time>`, full time as tooltip, one shared clock (`use_now`); `format_relative`, `format_duration`. |
| `LiveIndicator` | Live / connecting / offline; pulse only when live. |
| `SecretReveal` | A secret shown one time: warning, mono block, copy, "I saved it". |
| `CenterScreen` · `AuthCard` | Centred card for sign-in, callback and gate states. |
| Icons | `IconPlay` `IconPause` `IconBolt` `IconCopy` `IconClose` `IconChevron` `IconExternal` `IconCheck` `IconAlert` `IconInfo` `IconMenu` `IconSearch` `IconSun` `IconMoon` `IconMonitor` (`currentColor`). |
| `Dag` · `DagLegend` | The Aurora graph: layout (ranked DAG, or fixed layers with lanes) and interaction (select, open, hover, keyboard, "+N"). The product gives `DagNode` / `DagEdge` / `DagLane`. |
| `Graph` | The old graph API, a thin wrapper over `Dag`. |

Removed in 0.4 (no product used them): `NodeReadiness`, `InputTable`,
`StaleInputsBanner`, `StateCounts`, the `Input` model, `format_ago`,
`is_stale`, `freshness_pct`, `STALE_MS`.

### Pure logic (`tokens.rs`, `data.rs`)
`data.rs`: `format_relative`, `format_duration`, `page_range` /
`page_range_label` / `offset_for_limit`, `SortState` / `SortDir`,
`spark_points` / `spark_line_path` / `spark_area_path` / `spark_bars`,
`segment_widths`, `log_text`. `components.rs`: `menu_key_target`.

Semantic palette (`token::*`), `status_color`, `pill_bg` / `fill_for` /
`fg_for` / `tint`, and `ApiError` classification. `theme.rs` holds the `Theme`
type and `THEME_INIT_SCRIPT`. Framework-agnostic Rust — the seam where typed API models plug in.

## Built downstream (not shipped by the pack)
App branding (e.g. a logo mark) and app-specific state vocab/colors are supplied
by the consuming app as data. App panels around a graph (side lists, node
detail views) are built downstream. The graph layout and its interaction are
Aurora's (`graph_layout.rs`, a pure Sugiyama-style layout, and `Dag`).
