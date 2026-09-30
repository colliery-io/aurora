# Aurora — patterns & usage guide

How to choose and compose `aurora-leptos` pieces. Written for both people and AI
agents building Colliery Leptos UIs. Read **Core model** first, then use the
**Pick by intent** table to jump to a component, and **Choosing between similar
pieces** when two options look alike.

---

## Core model (read first)

1. **One stylesheet.** Load it once, one of two ways (see the README's "Styling"
   for setup): runtime injection via `<AuroraStyles/>` (simplest; possible flash),
   or a `<link>`ed `style/aurora.css` for no flash (emit it from `build.rs` under
   cargo-leptos, or a trunk `pre_build` hook). Without one, components render unstyled.
2. **Light and dark.** Surfaces/text/accents come from CSS custom properties
   (`--bg`, `--panel`, `--fg`, `--ice`, …). Each has a light and a dark value.
   The page follows the OS; `data-theme="light"`/`"dark"` on `<html>` forces
   one. Put `ThemeToggle` in the top bar and `THEME_INIT_SCRIPT` in the
   `<head>` (see the README, "Light and dark theme").
3. **Use tokens, not raw colours.** In Rust use `token::ICE` etc. (they are
   `var(--ice)` strings); in CSS use `var(--ice)`. Status meaning comes from
   `status_color(&str)`. Never hardcode a hex value: it does not change with the
   theme, and `tests/no_raw_colours.rs` refuses it in this crate. Do not add hex
   alpha to a token; use `tint(token::ICE, 12)` or `fill_for(token::ICE)`.
4. **The pack renders; the app supplies meaning.** Data-driven values — a status
   color, a state label, a tooltip string, a brand mark — are passed in as props.
   The pack ships no app vocabulary or logo. (e.g. a `HealthPill` takes
   `label`/`color`/`tip`; *you* decide that "live" is green.)
5. **Renderer-agnostic.** The crate depends on `leptos` with no renderer feature;
   your binary selects `csr` (or `hydrate`/`ssr`).
6. **Reactivity.** Inputs bind to `RwSignal`s (two-way); handlers are `Callback`s.
   `Modal`/`Drawer`/`ConfirmDialog`/`Menu` open-state are `RwSignal<bool>`.

```rust
use aurora_leptos::{AuroraStyles, components::*, widgets::*, graph::*, tokens::token};
```

---

## Pick by intent

| I want to… | Use | Notes |
|---|---|---|
| Lay out a horizontal row | `Group` | `gap`, `justify="between"`, `wrap` |
| Stack things vertically | `Stack` | `gap`, `center` |
| Equal-width grid | `SimpleGrid` | `cols=N` |
| Proportional 12-col grid | `Grid` + `GridCol span=` | when columns differ in width |
| App scaffold (nav + header + body) | `AppShell` + `SideNav` | fills the page; drawer below 768px |
| Sidebar links, groups, counts | `SideNav` + `SideNavGroup` + `SideNavLink` | `active` closure sets `aria-current` |
| Page title + subtitle + actions | `PageHeader` | `back_href`, `meta`, `actions` (or `right`) |
| Switch between views of one thing | `Tabs` + `TabPanel` | signal tabs, or `href` route tabs |
| A clickable card in a grid | `Card` | `href` or `on_click` |
| A bordered content card | `Panel` | `title` + optional `caption` |
| Body / caption / mono text | `Text` | `mono`, `dimmed`, `bright`, `bold`, `size` |
| Inline code / a link | `Code` / `Anchor` | — |
| A button | `Button` | `variant`, `size`, `bad`, `on_click`; `disabled=move \|\| busy.get()`; `loading` |
| A button that waits for a request | `Button loading=busy loading_label="Saving…"` | spinner, disabled, `aria-busy` |
| An icon-only button | `ActionIcon` | give it a `title` |
| Text / number / password / multiline field | `TextInput` / `NumberInput` / `PasswordInput` / `Textarea` | bind `value` to an `RwSignal`; `name`, `autocomplete`, `required`, `on_change` |
| A sign-in form | `CenterScreen` + `AuthCard` + `TextInput input_type="email"` + `PasswordInput autocomplete="current-password"` | a real `<form>` around it |
| Pick one of a few options | `Select` (many) · `SegmentedControl` (2–4) | `option_pairs` when the text is not the value |
| An on/off toggle | `Switch` | `checked: RwSignal<bool>` |
| Copy-to-clipboard | `CopyButton` | `value`; `icon`; `link` for a path |
| Show a new secret one time | `SecretReveal` in a `Modal` | `close_on_scrim=false`; clear it in `on_done` |
| A hover explanation | `Tooltip` (keyboard too) · `title=` (a plain hint) | see comparison below |
| A focused task over the page | `Modal` | `size` sm/md/lg/xl, `footer` |
| A detail view from the side | `Drawer` | same props as `Modal` |
| Confirm a destroy / cascade | `ConfirmDialog` | `impacts`, `confirm_text`, `busy` |
| Tell the person something happened | `use_toaster().success(…)` | mount one `ToastStack` |
| A dropdown of actions | `Menu` + `MenuItem` | `trigger` for your own trigger; `align="end"`; `up` |
| A short status tag | `StatusBadge` (status string) · `Pill` (custom hue) · `HealthPill` (+tooltip) | see comparison below |
| A small status dot | `Dot` | `color`, `glow` |
| A filter toggle | `Chip` | `active: Signal<bool>` |
| Let the user choose light / dark / system | `ThemeToggle` | call `provide_theme()` once at the root |
| Loading / empty / error states | `Loading` / `Empty` / `ErrorState` | every async view should use these |
| An inline notice / callout | `Banner` (transient) · `Alert` (in-content) | — |
| Counts per state | `SegmentedBar legend=true` · `Pill`s | `Segment::new(label, n, token::OK)` |
| A KPI number | `StatTile` (+ `Sparkline` in `spark`) | `color`, `unit`, `delta`, `sub` |
| A small trend | `Sparkline` | line, `fill`, or `bars`; `currentColor` |
| Label / value details | `DetailList` + `KeyValue` | `mono`, `stacked` |
| A section heading in a panel | `SectionLabel` | `count`, `action` |
| Code, config, a manifest | `CodeBlock` | `max_height`, `wrap`, copy |
| A log or an event stream | `LogView` + `LogLine` | follows the tail, copy |
| An activity or run feed | `FeedList` + `FeedRow` | `at`, `subject`, `status`, `href` / `on_click` |
| "3m ago" | `RelativeTime` | `at` (ms) or `iso`; `format_duration` for durations |
| The state of a live stream | `LiveIndicator` | `LiveState::{Live, Connecting, Offline}` |
| Pages of a long list | `Pagination` | `offset`, `limit`, `total`, `page_sizes` |
| An icon | `IconPlay`, `IconCopy`, ... | `size`, `title` |
| A freshness / progress bar | `Meter` | `value` 0–100 |
| Build/CI status | `BuildStatusBadge` | success/building/failed/pending |
| A data table | `Table` + `TableRow` / `SortHeader` / `TableEmpty` | `fixed`, `widths`, `min_width` |
| Draw a graph / DAG | `Dag` (+ `DagLegend`) | Aurora does the layout and the interaction; old `Graph` still works |

---

## Choosing between similar pieces

- **`StatusBadge` vs `Pill` vs `HealthPill` vs `Chip`** — `StatusBadge` when you
  have a status *string* and want the standard `status_color` mapping
  (running/completed/failed/…). `Pill` when you choose the hue yourself
  (`color=token::VIOLET`). `HealthPill` when the tag needs a hover tooltip
  explaining the state. `Chip` only for interactive *filters* (it's clickable).
- **`Banner` vs `Alert` vs `ErrorState`** — `ErrorState` for a failed async load
  (takes an `ApiError`, renders the right title/retry by kind). `Alert` for an
  in-content callout tied to a section. `Banner` for a page-level transient notice
  (full-width, accent + icon).
- **`Tooltip` vs `title=`** — `title=` is enough for a plain hint on truncated
  text or an icon that already has an `aria-label`. Use `Tooltip` when the
  text matters to keyboard users (it shows on focus) or must show fast and in
  a set place. Inside an SVG chart, use an SVG `<title>`.
- **`Meter` vs `SegmentedBar` vs `Sparkline`** — `Meter`: one value from 0 to
  100. `SegmentedBar`: parts of a whole. `Sparkline`: a value over time.
- **`CodeBlock` vs `LogView` vs `Code`** — `Code` is inline. `CodeBlock` is a
  block of text that does not change. `LogView` is lines that come in.
- **`FeedList` vs `Table`** — a feed is a time-ordered list of events with one
  line each. A table has columns to compare and sort.
- **`SimpleGrid` vs `Grid`** — `SimpleGrid cols=N` for equal columns (cards,
  stats). `Grid` + `GridCol span=` (out of 12) when widths differ.
- **`Group`/`Stack` vs grids** — `Group`/`Stack` for a handful of inline items
  (flex); grids for tabular/cellular layouts that should wrap evenly.
- **`Modal` vs `Menu`** — `Modal` for a focused task/confirmation that blocks the
  page; `Menu` for a small list of actions off a trigger button.
- **`Modal` vs `Drawer` vs `ConfirmDialog`** — `Modal` for a form or a task.
  `Drawer` for a detail view that keeps the list in sight. `ConfirmDialog` for
  every action that deletes, archives or cascades: never delete on one click.
- **`Tabs` vs `SegmentedControl`** — `Tabs` change the view (an underline tab
  list, ARIA `tablist`). `SegmentedControl` picks a value for a filter or a
  mode (a boxed control, `aria-pressed`).
- **`Card` vs `Panel`** — `Panel` is a titled section of a page. `Card` is one
  of many things in a grid; it can be a link or a button.
- **Toast vs `Banner` vs `Alert`** — a toast is a short message that goes away
  (saved, deleted, failed to send). A `Banner` or an `Alert` stays on the page
  while the state is true.

---

## Reference by area

Key props only — see rustdoc for the full signatures.

### Layout
`Box` · `Group{justify,top,wrap,gap}` · `Stack{center,gap}` ·
`SimpleGrid{cols}` · `Grid` + `GridCol{span}` · `Divider` ·
`Card{href,on_click,title,label,selected}`.

### Frame
`AppShell{header?, navbar?, brand?, contained, menu_label}` ·
`SideNav{label, footer?}` · `SideNavGroup{label}` ·
`SideNavLink{href, active, count, count_color, marker, icon?, on_click}` ·
`PageHeader{title, sub, back_href, back_label, meta?, actions?, right?}` ·
`Tabs{tabs:Vec<TabItem>, value, label}` + `TabPanel{value}`.
Icons: `IconPlay` `IconPause` `IconBolt` `IconCopy` `IconClose` `IconChevron{dir}`
`IconExternal` `IconCheck` `IconAlert` `IconInfo` `IconMenu` `IconSearch`
`IconSun` `IconMoon` `IconMonitor`, each `{size, title}`.

```rust
let path = use_location().pathname; // or your own route signal
view! {
    <AppShell
        brand=Arc::new(|| view! { <Logo/> "Cloacina" }.into_any())
        navbar=Box::new(move || view! {
            <SideNav footer=Box::new(|| view! { <TenantSwitcher/> }.into_any())>
                <SideNavLink href="/" active=move || path.get() == "/">"Overview"</SideNavLink>
                <SideNavGroup label="Orchestration">
                    <SideNavLink href="/workflows" count=3usize
                        active=move || path.get().starts_with("/workflows")>"Workflows"</SideNavLink>
                </SideNavGroup>
            </SideNav>
        }.into_any())
    >
        <Outlet/>
    </AppShell>
}
```

`AppShell` fills the page. With no `header`, the `brand` shows at the top of
the sidebar; with a `header`, it shows in the header. Below 768px the sidebar
is a drawer behind a menu button (Escape, the scrim or a link closes it).

### Typography
`Text{size,dimmed,bright,bold,mono}` · `Code` · `Anchor{href}` ·
`List` + `ListItem`. The `MONO`/tabular look comes from `mono=true` or the
`.cl-mono` class.

### Inputs (bind `value`/`checked` to an `RwSignal`)
`Button{variant,size,bad,disabled,loading,loading_label,button_type,href,title,aria_label,stop_propagation,on_click}` ·
`ActionIcon{title,on_click}` ·
`TextInput{label,placeholder,value,error,input_type,disabled,on_input,on_change,name,autocomplete,required,spellcheck,mono}` ·
`Textarea{…,rows,mono}` · `PasswordInput{…,autocomplete}` ·
`NumberInput{label,value:RwSignal<f64>,step,min,max,disabled,on_change,name,required,error}` ·
`Select{label,options,option_pairs,value,placeholder,disabled,on_change,name,required,error}` ·
`Switch{checked,label,disabled,on_change}` · `SegmentedControl{options,value}` ·
`CopyButton{value,icon,link,label,copied_label,on_copy}`.

`disabled` takes a `bool`, a signal or a closure, so it changes with the
state: `disabled=move || busy.get() || name.get().is_empty()`. An attribute
with no prop goes on the root element with `attr:`
(`<Button attr:data-testid="save">`).

```rust
let name = RwSignal::new(String::new());
view! { <TextInput label="Name" value=name placeholder="e.g. nightly" /> }
```

### Overlays & feedback
`Tooltip{label,position,focusable}` · `Modal{open,title,size,footer?,close_on_scrim,locked,on_close}` ·
`Drawer{…same}` · `ConfirmDialog{open,title,message,impacts,confirm_text,confirm_label,danger,busy,on_confirm,on_cancel}` ·
`ToastStack{duration_ms}` + `provide_toaster()` / `use_toaster()` ·
`Menu{label,trigger?,trigger_class,aria_label,align,up,open}` + `MenuItem{on_click,href,disabled,danger}` + `MenuLabel` + `MenuDivider` ·
`SecretReveal{secret,label,warning,done_label,on_done}` · `CenterScreen{contained}` + `AuthCard{title,sub,brand?,footer?}` ·
`Alert{title,color}` · `Loader`.

Dialogs move focus in on open (to `data-autofocus`, else the first control),
keep Tab inside, close on Escape, and give focus back to the opener.

```rust
let open = RwSignal::new(false);
view! {
    <Button on_click=Callback::new(move |_| open.set(true))>"Open"</Button>
    <Modal open=open title="Confirm"> <Text>"…"</Text> </Modal>
}

// A destroy with a cascade and a name check. It stays open until you close it.
view! {
    <ConfirmDialog open=del title="Delete workflow?"
        message="This deletes the workflow and its run history."
        impacts=children_codes            // Signal<Vec<String>> or Vec<String>
        confirm_text="nightly-ingest" confirm_label="Delete" busy=deleting
        on_confirm=Callback::new(move |_| start_delete()) />
}

// Toasts: once at the root, then from anywhere.
provide_toaster();                        // + <ToastStack/> in the root view
let toaster = use_toaster();              // Copy: capture it for async code
toaster.success("Workflow deployed");
toaster.toast(ToastKind::Error, "The server did not answer");
```

### Status & async states
`Pill{color}` · `StatusBadge{status}` · `Dot{color,size,glow}` ·
`Chip{label,count,active,on_click}` · `Loading{label}` · `Empty{message}` ·
`ErrorState{error:ApiError,on_retry}`.

```rust
// Async view shape: never a blank screen.
match resource.get() {
    None => view! { <Loading/> }.into_any(),
    Some(Err(e)) => view! { <ErrorState error=e on_retry=retry/> }.into_any(),
    Some(Ok(items)) if items.is_empty() => view! { <Empty message="Nothing here."/> }.into_any(),
    Some(Ok(items)) => /* render */,
}
```

### Data-display widgets
`Meter{value,color,label}` · `Banner{color,icon}` · `HealthPill{label,color,tip}` ·
`BuildStatusBadge{status}`.

### Data components
`StatTile{label,value,unit,sub,delta,delta_color,color,spark?}` ·
`Sparkline{values,bars,fill,width,height,fluid,zero_base,color,label}` ·
`SegmentedBar{segments:Vec<Segment>,legend,label,height}` ·
`DetailList{mono,dividers,stacked,label_width}` + `KeyValue{label,mono}` ·
`SectionLabel{label,count,action?,divider,level}` ·
`CodeBlock{code,max_height,copy,wrap,label}` ·
`LogView{lines:Vec<LogLine>,max_height,follow,copy,empty,label}` ·
`FeedList{label}` + `FeedRow{at,time,subject,actor,status,status_color,dot,href,on_click}` ·
`Pagination{offset,limit,total,page_sizes,prev_label,next_label,on_change}` ·
`RelativeTime{at,iso,fallback}` · `LiveIndicator{state,live_label,connecting_label,offline_label,compact}` ·
`Table{mono,fixed,widths,min_width,label}` + `TableRow{on_click,selected,label}` +
`SortHeader{label,key,sort,first_desc,align_right,width}` + `TableEmpty{message,colspan}`.

```rust
// A sortable, clickable, paged table.
let sort = RwSignal::new(SortState::by("started", SortDir::Desc));
let (offset, limit) = (RwSignal::new(0), RwSignal::new(20));
view! {
    <Table fixed=true widths=vec!["30%".into(), "20%".into(), "50%".into()]>
        <thead><tr>
            <SortHeader label="Name" key="name" sort=sort />
            <SortHeader label="Started" key="started" sort=sort first_desc=true />
            <th>"Status"</th>
        </tr></thead>
        <tbody>
            {move || rows_for(sort.get(), offset.get(), limit.get()).into_iter().map(|r| view! {
                <TableRow on_click=Callback::new(move |_| open(r.id))>
                    <td>{r.name}</td><td><RelativeTime at=r.started /></td><td><StatusBadge status=r.status /></td>
                </TableRow>
            }).collect_view()}
        </tbody>
    </Table>
    <Pagination offset limit total=total page_sizes=vec![20, 50, 100] />
}
```

The pure helpers are in `data.rs` and have tests: `format_relative(ms)`
("3m ago", "in 2h"), `format_duration(ms)` ("4.2s", "3m 05s"),
`page_range(offset, limit, total)` + `page_range_label`, `SortState::toggle`,
`spark_points`, `segment_widths`. `use_now()` is the shared clock (one timer,
one second).

### Graph / DAG
Aurora owns the layout and the interaction of the graph. The product gives
only the data. Use `Dag`:

- `DagNode::new(id, label)` with `.sublabel()`, `.detail()`, `.status(Hue::Ok)`,
  `.layer(n)` (a fixed column), `.lane(id)` (a containment band),
  `.current()`, `.archived()`, `.done()`, `.mark("done", Hue::Ok)`, `.more(n)`
  (a "+N" badge), `.sort_key()`.
- `DagEdge::new(from, to).style("ok")`. The product names the styles:
  `EdgeStyle::new("ok", "Succeeded", Hue::Ok)`, `.dashed()`, `.animated()`.
- Optional `DagLane::new(id, label).anchor(parent_id)` and `layers` (column
  headers).

```rust
let selected = RwSignal::new(None::<String>);
let nodes = vec![
    DagNode::new("fetch", "fetch").status(Hue::Ok),
    DagNode::new("load", "load").status(Hue::Ice),
];
let edges = vec![DagEdge::new("fetch", "load").style("ok")];
let styles = vec![EdgeStyle::new("ok", "Succeeded", Hue::Ok)];
view! {
    <Dag nodes edges styles legend=true
         on_select=Callback::new(move |id: String| selected.set(Some(id)))
         on_open=Callback::new(move |id: String| navigate_to(&id)) />
}
```

Two shapes:
- **Ranked DAG** (a workflow): give no `layer`. The ranks come from the edges,
  left to right (`direction=Direction::TopBottom` for rows).
- **Fixed layers with lanes** (flight levels): give each node a `layer` and a
  `lane`, the lanes an `anchor`, and `layers=vec!["Strategy".into(), ...]`.
  Use `align=Align::Start` for stacked columns. Containment is a lane, never
  an edge.

Interaction: click, Space, or Enter selects (`on_select`). Double click, or
Enter on the selected node, opens (`on_open`). Hover or keyboard focus
highlights the edges of a node and dims the rest. Tab moves through the
nodes by layer. The "+N" badge calls `on_more`. Give `selected` to hold the
selection in the product.

Status is a class (`status-ok`, ...) from the `--x` / `--x-fg` / `--x-bg`
tokens. Arrowheads use one marker per style, from the same tokens, so both
themes work. `DagLegend styles=...` lists styles alone; `legend=true` shows
the styles that the edges use.

The layout is `graph_layout::layout`, a pure function (no DOM). Call it to
draw the geometry in another way. It is deterministic, and the order of the
input nodes does not change it. Limits: one lane per node, one size for all
nodes, no self loops.

The old API, `Graph{nodes:Vec<GraphNode>, edges:Vec<GraphEdge>, direction}`
(`"TB"` default, or `"LR"`), still works. It is a thin wrapper over `Dag`: a
`GraphNode` colour token becomes the status of the same hue, and
`GraphEdge::active` becomes the `active` style.

---

## Tokens

- **Surfaces**: `--bg --sidebar --panel --panel-2 --inset --field --control --control-hover --border --border-soft --border-fainter --border-control --edge`
- **Text**: `--fg --fg-bright --fg-2 --fg-strong --muted --faint --fainter` (`--fainter`: large text, disabled and decoration only)
- **Accents/status**: `--ice --teal --violet --gold --ok --bad --skip --muted` (Rust: `token::*`)
- **Status pairs**: `--x-fg` (text) on `--x-bg` (fill) for each hue above (Rust: `token::X_FG`, `token::X_BG`); `--on-status` for text on a solid hue
- **Overlays**: `--tooltip-bg --tooltip-fg --tooltip-border --scrim --shadow-sm --shadow-md --shadow-lg --scrollbar`
- **Scales**: `--space-{xs..xl}`, `--radius-{xs..xl}` + `--radius-pill/panel/chip`,
  `--fs-{xs..xl}`, `--h-{xs,sm,md}` (control heights), `--font-sans`/`--font-mono`.

Data-driven color goes inline; static chrome uses the `.cl-*` classes. To color a
status, call `status_color(s)` (or your own map) and pass it as a prop.

---

## Checklist for agents

1. Add the dep + a renderer feature in the binary; `use aurora_leptos::…`.
2. Render `<AuroraStyles/>` once (or link the CSS files).
3. Reach for an existing component via **Pick by intent** before writing markup.
4. Bind inputs to `RwSignal`s; pass handlers as `Callback`s.
5. Use `token::*` / `var(--…)` and `status_color` — never invent classes or raw colours.
6. Supply app-specific labels/colors/branding as **data**; don't add them to the pack.
7. Wrap async UI in `Loading`/`Empty`/`ErrorState`.
