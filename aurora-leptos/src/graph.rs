//! The Aurora graph: layout and interaction owned by Aurora, data from the
//! product.
//!
//! - [`Dag`] draws a layered graph from [`DagNode`]s, [`DagEdge`]s and
//!   optional [`DagLane`]s. The layout is [`graph_layout::layout`], a pure
//!   function. It supports a ranked DAG (ranks from the edges) and fixed
//!   layers with lane bands (containment).
//! - Interaction: click selects (`on_select`), double click or Enter on the
//!   selected node opens (`on_open`). With `defer_select`, a click waits for
//!   a possible second click, so a double click opens without a select. Hover or keyboard focus highlights the
//!   edges of a node and dims the rest. Nodes are in the tab order, by layer.
//! - The status of a node is a class (`status-ok`, ...) that maps to the
//!   `--x` / `--x-fg` / `--x-bg` tokens. Edge styles are named by the product;
//!   their arrowheads use the same tokens, so both themes work.
//! - [`DagLegend`] shows the edge styles.
//! - [`Graph`] is the old API (`GraphNode`, `GraphEdge`). It stays, as a thin
//!   wrapper over [`Dag`].

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use leptos::prelude::*;

use crate::graph_layout;
pub use crate::graph_layout::{
    default_edge_styles, Align, DagEdge, DagLane, DagNode, Direction, EdgeStyle, Hue,
    LayoutOptions, NodeMark, DEFAULT_STYLE,
};
use crate::tokens::token;

/// A number for the ids of the arrowhead markers, unique on the page.
static NEXT_GRAPH: AtomicUsize = AtomicUsize::new(0);

/// Line height of the text in a node.
const LINE: f64 = 14.0;
/// Inner padding of a node.
const PAD: f64 = 10.0;

/// `text` cut to fit `width` pixels at `char_w` pixels a character, with an
/// ellipsis. SVG text does not wrap.
fn clip(text: &str, width: f64, char_w: f64) -> String {
    let max = (width / char_w).floor().max(1.0) as usize;
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

/// The style of each name, with the built-in styles under the product's.
fn style_table(styles: Option<Vec<EdgeStyle>>) -> Vec<EdgeStyle> {
    let mut table = styles.unwrap_or_default();
    for builtin in default_edge_styles() {
        if !table.iter().any(|s| s.name == builtin.name) {
            table.push(builtin);
        }
    }
    table
}

/// The styles that `edges` use, in the order of `table`. An unknown style
/// name gets a neutral style labelled with the name.
fn used_styles(table: &[EdgeStyle], edges: &[DagEdge]) -> Vec<EdgeStyle> {
    let names: HashSet<&str> = edges.iter().map(|e| e.style.as_str()).collect();
    let mut used: Vec<EdgeStyle> = table
        .iter()
        .filter(|s| names.contains(s.name.as_str()))
        .cloned()
        .collect();
    let mut unknown: Vec<&str> = names
        .into_iter()
        .filter(|n| !table.iter().any(|s| s.name == *n))
        .collect();
    unknown.sort_unstable();
    for name in unknown {
        used.push(EdgeStyle::new(name, name, Hue::Neutral));
    }
    used
}

/// `name` as a part of an id: ASCII letters and digits; any other run of
/// characters is one `-`.
fn id_part(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "x".to_string()
    } else {
        out
    }
}

/// The id of the arrowhead marker of edge style `style`, in the graph with
/// the id `base`. It depends only on the two names, so a product or a test
/// can find it (AURORA-T-0007 item 15).
pub fn dag_marker_id(base: &str, style: &str) -> String {
    format!("{base}-arrow-{}", id_part(style))
}

/// What a click on a node does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickIntent {
    /// Select the node now.
    SelectNow,
    /// Select it when no second click comes (`defer_select`).
    SelectLater,
    /// Nothing: the second click of a double click (the `dblclick` opens).
    Ignore,
}

/// What a node click with the click count `detail` (the `detail` of the DOM
/// event: 1, then 2 for the second click of a double click) does
/// (AURORA-T-0007 item 16).
pub fn click_intent(detail: i32, defer_select: bool) -> ClickIntent {
    if detail >= 2 {
        ClickIntent::Ignore
    } else if defer_select {
        ClickIntent::SelectLater
    } else {
        ClickIntent::SelectNow
    }
}

/// How long a deferred select waits for a second click, in ms.
pub const DOUBLE_CLICK_MS: u64 = 250;

fn edge_classes(style: &EdgeStyle) -> String {
    let mut c = format!("cl-dag__edge cl-dag__hue--{}", style.hue.name());
    if style.dashed {
        c.push_str(" cl-dag__edge--dashed");
    }
    if style.animated {
        c.push_str(" cl-pulse");
    }
    c
}

/// A layered graph. Aurora does the layout and the interaction; the product
/// gives the data.
///
/// ```ignore
/// let nodes = vec![
///     DagNode::new("fetch", "fetch").status(Hue::Ok),
///     DagNode::new("load", "load").status(Hue::Ice),
/// ];
/// let edges = vec![DagEdge::new("fetch", "load").style("ok")];
/// let styles = vec![EdgeStyle::new("ok", "Succeeded", Hue::Ok)];
/// view! {
///     <Dag nodes edges styles legend=true
///          on_select=Callback::new(move |id: String| selected.set(Some(id))) />
/// }
/// ```
#[component]
pub fn Dag(
    nodes: Vec<DagNode>,
    edges: Vec<DagEdge>,
    /// Lane labels and anchors. A node's `lane` works without one.
    #[prop(optional)]
    lanes: Vec<DagLane>,
    /// Layer headers (column labels), in layer order.
    #[prop(optional)]
    layers: Vec<String>,
    /// The edge styles. The built-in `default` and `active` are added when
    /// not named here.
    #[prop(optional)]
    styles: Option<Vec<EdgeStyle>>,
    /// Sizes and gaps. `direction`, `align`, `node_w` and `node_h` override it.
    #[prop(optional)]
    options: Option<LayoutOptions>,
    #[prop(optional)] direction: Option<Direction>,
    #[prop(optional)] align: Option<Align>,
    #[prop(optional)] node_w: Option<f64>,
    #[prop(optional)] node_h: Option<f64>,
    /// The selected node id, when the product holds the selection. Without
    /// it, the graph holds the selection itself.
    #[prop(optional, into)]
    selected: Option<Signal<Option<String>>>,
    /// Click, Space, or Enter on a node that is not selected.
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    /// Double click, or Enter on the selected node.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
    /// Click or Enter on the "+N" badge of a node.
    #[prop(optional)]
    on_more: Option<Callback<String>>,
    /// Show a legend of the edge styles that the graph uses.
    #[prop(optional)]
    legend: bool,
    /// The accessible name of the graph.
    #[prop(optional, into)]
    label: Option<String>,
    /// `true` (default): shrink to the width of the container. `false`:
    /// natural size, with scroll bars.
    #[prop(default = true)]
    fit: bool,
    /// The id of the `<svg>`, and the start of the marker ids
    /// (`{id}-arrow-{style}`, see [`dag_marker_id`]). Default: a unique
    /// `cl-dag-N`.
    #[prop(optional, into)]
    id: String,
    /// A click waits [`DOUBLE_CLICK_MS`] before `on_select`; a double click
    /// in that time runs only `on_open`. Use it when a select navigates
    /// away. Keyboard select does not wait.
    #[prop(optional)]
    defer_select: bool,
) -> impl IntoView {
    let mut opts = options.unwrap_or_default();
    if let Some(d) = direction {
        opts.direction = d;
    }
    if let Some(a) = align {
        opts.align = a;
    }
    if let Some(w) = node_w {
        opts.node_w = w;
    }
    if let Some(h) = node_h {
        opts.node_h = h;
    }
    let geometry = graph_layout::layout(&nodes, &edges, &lanes, &layers, &opts);
    let base_id = if id.is_empty() {
        format!("cl-dag-{}", NEXT_GRAPH.fetch_add(1, Ordering::Relaxed))
    } else {
        id
    };

    let table = style_table(styles);
    let used = used_styles(&table, &edges);
    let marker_of: HashMap<String, String> = used
        .iter()
        .map(|s| (s.name.clone(), dag_marker_id(&base_id, &s.name)))
        .collect();
    let style_of = |name: &str| {
        used.iter()
            .find(|s| s.name == name)
            .cloned()
            .unwrap_or_else(|| EdgeStyle::new(name, name, Hue::Neutral))
    };

    // ---- interaction state --------------------------------------------------
    let hovered = RwSignal::new(None::<String>);
    let own_selection = RwSignal::new(None::<String>);
    let is_selected = move |id: &str| match selected {
        Some(s) => s.with(|s| s.as_deref() == Some(id)),
        None => own_selection.with(|s| s.as_deref() == Some(id)),
    };
    let select = move |id: String| {
        own_selection.set(Some(id.clone()));
        if let Some(cb) = on_select {
            cb.run(id);
        }
    };
    let open = move |id: String| {
        if let Some(cb) = on_open {
            cb.run(id);
        }
    };
    // A select that waits for a possible second click (`defer_select`).
    let pending = StoredValue::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    let cancel_pending = move || {
        if let Some(h) = pending.try_update_value(|p| p.take()).flatten() {
            h.clear();
        }
    };
    let mut neighbours: HashMap<String, HashSet<String>> = HashMap::new();
    for e in &geometry.edges {
        neighbours
            .entry(e.from.clone())
            .or_default()
            .insert(e.to.clone());
        neighbours
            .entry(e.to.clone())
            .or_default()
            .insert(e.from.clone());
    }
    let neighbours = Arc::new(neighbours);

    // ---- defs: one arrowhead per used style --------------------------------
    let markers = used
        .iter()
        .map(|s| {
            view! {
                <marker
                    id=marker_of[&s.name].clone()
                    data-style=s.name.clone()
                    viewBox="0 0 10 10"
                    refX="9"
                    refY="5"
                    markerWidth="8"
                    markerHeight="8"
                    markerUnits="userSpaceOnUse"
                    orient="auto-start-reverse"
                >
                    <path
                        class=format!("cl-dag__arrowhead cl-dag__hue--{}", s.hue.name())
                        d="M 0 0 L 10 5 L 0 10 z"
                    />
                </marker>
            }
        })
        .collect_view();

    // ---- lanes and headers --------------------------------------------------
    let lane_views = geometry
        .lanes
        .iter()
        .map(|lane| {
            view! {
                <g class="cl-dag__lane-group" data-lane=lane.id.clone()>
                    <rect
                        class="cl-dag__lane"
                        x=lane.x
                        y=lane.y
                        width=lane.w
                        height=lane.h
                        rx="10"
                    />
                    <text class="cl-dag__lane-label" x=lane.label_x y=lane.label_y>
                        {clip(&lane.label, lane.w - 12.0, 6.0)}
                    </text>
                </g>
            }
        })
        .collect_view();
    let lr = opts.direction == Direction::LeftRight;
    let header_views = geometry
        .headers
        .iter()
        .map(|h| {
            let baseline = if lr { "auto" } else { "middle" };
            view! {
                <text class="cl-dag__header" x=h.x y=h.y dominant-baseline=baseline>
                    {h.label.clone()}
                </text>
            }
        })
        .collect_view();

    // ---- edges ---------------------------------------------------------------
    let edge_views = geometry
        .edges
        .iter()
        .map(|placed| {
            let edge = &edges[placed.index];
            let style = style_of(&edge.style);
            let base = edge_classes(&style);
            let (from, to) = (placed.from.clone(), placed.to.clone());
            let (data_from, data_to) = (from.clone(), to.clone());
            let tip = edge
                .label
                .clone()
                .unwrap_or_else(|| format!("{}: {} → {}", style.label, from, to));
            let class = move || {
                let hot = hovered.with(|h| h.as_deref().is_some_and(|h| h == from || h == to));
                if hot {
                    format!("{base} cl-dag__edge--hot")
                } else {
                    base.clone()
                }
            };
            view! {
                <path
                    class=class
                    data-style=edge.style.clone()
                    data-from=data_from
                    data-to=data_to
                    d=placed.path.clone()
                    marker-end=format!("url(#{})", marker_of[&style.name])
                >
                    <title>{tip}</title>
                </path>
            }
        })
        .collect_view();

    // ---- nodes ---------------------------------------------------------------
    let node_views = geometry
        .nodes
        .iter()
        .map(|placed| {
            let node = nodes[placed.index].clone();
            let (x, y, w, h) = (placed.x, placed.y, placed.w, placed.h);
            let id = node.id.clone();

            // Static classes.
            let mut base = format!("cl-dag__node {}", node.status.status_class());
            if node.current {
                base.push_str(" cl-dag__node--current");
            }
            if node.archived {
                base.push_str(" cl-dag__node--archived");
            }
            if node.done {
                base.push_str(" cl-dag__node--done");
            }
            let class = {
                let id = id.clone();
                let neighbours = neighbours.clone();
                move || {
                    let mut c = base.clone();
                    if is_selected(&id) {
                        c.push_str(" cl-dag__node--selected");
                    }
                    hovered.with(|h| {
                        if let Some(h) = h.as_deref() {
                            if h == id {
                                c.push_str(" cl-dag__node--hot");
                            } else if neighbours.get(h).is_some_and(|n| n.contains(&id)) {
                                c.push_str(" cl-dag__node--near");
                            }
                        }
                    });
                    c
                }
            };
            let pressed = {
                let id = id.clone();
                move || if is_selected(&id) { "true" } else { "false" }
            };

            // Text lines: label, sublabel, detail. Marks sit at the right
            // end of the last line.
            let marks_w: f64 = node
                .marks
                .iter()
                .map(|m| m.label.chars().count() as f64 * 6.0 + 8.0)
                .sum();
            let mut lines: Vec<(&'static str, String, f64)> =
                vec![("cl-dag__label", node.label.clone(), 7.0)];
            if let Some(s) = &node.sublabel {
                lines.push(("cl-dag__sublabel", s.clone(), 6.1));
            }
            if let Some(s) = &node.detail {
                lines.push(("cl-dag__detail", s.clone(), 5.9));
            }
            let count = lines.len();
            let first_y = y + h / 2.0 - (count as f64 - 1.0) * LINE / 2.0;
            let last_y = first_y + (count as f64 - 1.0) * LINE;
            let text_views = lines
                .into_iter()
                .enumerate()
                .map(|(i, (class, text, char_w))| {
                    let room = w - 2.0 * PAD - if i + 1 == count { marks_w } else { 0.0 };
                    view! {
                        <text
                            class=class
                            x=x + PAD
                            y=first_y + i as f64 * LINE
                            dominant-baseline="central"
                        >
                            {clip(&text, room, char_w)}
                        </text>
                    }
                })
                .collect_view();
            let mut mark_x = x + w - PAD;
            let mark_views = node
                .marks
                .iter()
                .map(|m| {
                    let at = mark_x;
                    mark_x -= m.label.chars().count() as f64 * 6.0 + 8.0;
                    view! {
                        <text
                            class=format!("cl-dag__mark cl-dag__hue--{}", m.hue.name())
                            x=at
                            y=last_y
                            dominant-baseline="central"
                        >
                            {m.label.clone()}
                        </text>
                    }
                })
                .collect_view();

            let mut aria = node.label.clone();
            for part in [&node.sublabel, &node.detail].into_iter().flatten() {
                aria.push_str(", ");
                aria.push_str(part);
            }
            for m in &node.marks {
                aria.push_str(", ");
                aria.push_str(&m.label);
            }
            let tooltip = node.tooltip.clone().unwrap_or_else(|| aria.clone());

            let (click_id, dbl_id, key_id, enter_id, focus_id, item_id) = (
                id.clone(),
                id.clone(),
                id.clone(),
                id.clone(),
                id.clone(),
                id.clone(),
            );
            let on_key = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
                "Enter" => {
                    ev.prevent_default();
                    if on_open.is_some() && is_selected(&key_id) {
                        open(key_id.clone());
                    } else {
                        select(key_id.clone());
                    }
                }
                " " | "Spacebar" => {
                    ev.prevent_default();
                    select(key_id.clone());
                }
                _ => {}
            };

            let more = (node.more > 0).then(|| {
                let n = node.more;
                let (more_id, more_key_id, more_node, more_focus_id) =
                    (id.clone(), id.clone(), id.clone(), id.clone());
                let (cx, cy) = (x + w - 2.0, y + 2.0);
                view! {
                    <g
                        class="cl-dag__more"
                        data-node=more_node
                        role="button"
                        tabindex="0"
                        aria-label=format!("Show {n} more linked items")
                        on:click=move |ev| {
                            ev.stop_propagation();
                            if let Some(cb) = on_more {
                                cb.run(more_id.clone());
                            }
                        }
                        on:dblclick=move |ev| ev.stop_propagation()
                        on:focus=move |_| hovered.set(Some(more_focus_id.clone()))
                        on:blur=move |_| hovered.set(None)
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if matches!(ev.key().as_str(), "Enter" | " " | "Spacebar") {
                                ev.prevent_default();
                                ev.stop_propagation();
                                if let Some(cb) = on_more {
                                    cb.run(more_key_id.clone());
                                }
                            }
                        }
                    >
                        <title>{format!("{n} more linked items")}</title>
                        <circle cx=cx cy=cy r="10" />
                        <text x=cx y=cy dominant-baseline="central">
                            {format!("+{n}")}
                        </text>
                    </g>
                }
            });

            let on_click =
                move |ev: leptos::ev::MouseEvent| match click_intent(ev.detail(), defer_select) {
                    ClickIntent::SelectNow => select(click_id.clone()),
                    ClickIntent::SelectLater => {
                        cancel_pending();
                        let id = click_id.clone();
                        let h = set_timeout_with_handle(
                            move || {
                                pending.set_value(None);
                                select(id);
                            },
                            std::time::Duration::from_millis(DOUBLE_CLICK_MS),
                        )
                        .ok();
                        pending.set_value(h);
                    }
                    ClickIntent::Ignore => {}
                };
            let on_dblclick = move |_| {
                cancel_pending();
                own_selection.set(Some(dbl_id.clone()));
                open(dbl_id.clone());
            };

            // The node and its "+N" badge are one item: the hover covers
            // both, the badge dims with its node, and `[data-node=id]`
            // finds both (AURORA-T-0007 item 17). The badge is not inside
            // the node's `role="button"`: a button in a button has no
            // accessible name of its own.
            view! {
                <g
                    class="cl-dag__item"
                    data-node=item_id
                    on:mouseenter=move |_| hovered.set(Some(enter_id.clone()))
                    on:mouseleave=move |_| hovered.set(None)
                >
                <g
                    class=class
                    data-id=id.clone()
                    data-kind=node.kind.clone()
                    role="button"
                    tabindex="0"
                    aria-label=aria
                    aria-pressed=pressed
                    on:click=on_click
                    on:dblclick=on_dblclick
                    on:keydown=on_key
                    on:focus=move |_| hovered.set(Some(focus_id.clone()))
                    on:blur=move |_| hovered.set(None)
                >
                    <title>{tooltip}</title>
                    <rect
                        class="cl-dag__ring"
                        x=x - 4.0
                        y=y - 4.0
                        width=w + 8.0
                        height=h + 8.0
                        rx="12"
                    />
                    <rect class="cl-dag__box" x=x y=y width=w height=h rx="8" />
                    <rect class="cl-dag__tint" x=x y=y width=w height=h rx="8" />
                    {text_views}
                    {mark_views}
                </g>
                {more}
                </g>
            }
        })
        .collect_view();

    let legend_view = legend.then(|| view! { <DagLegend styles=used.clone() /> });
    let (w, h) = (geometry.width, geometry.height);
    let svg_class = move || {
        if hovered.with(Option::is_some) {
            "cl-dag cl-dag--hovering"
        } else {
            "cl-dag"
        }
    };
    let wrap_class = if fit {
        "cl-dag-wrap"
    } else {
        "cl-dag-wrap cl-dag-wrap--scroll"
    };
    view! {
        <div class=wrap_class>
            {legend_view}
            <svg
                id=base_id
                class=svg_class
                width=w
                height=h
                viewBox=format!("0 0 {w} {h}")
                role="group"
                aria-label=label.unwrap_or_else(|| "Graph".to_string())
            >
                <defs>{markers}</defs>
                {lane_views}
                {header_views}
                <g class="cl-dag__edges">{edge_views}</g>
                <g class="cl-dag__nodes">{node_views}</g>
            </svg>
        </div>
    }
}

/// A legend of edge styles: a sample line with its arrowhead and the label
/// of each style. [`Dag`] shows one with `legend=true` (only the styles that
/// its edges use); use this component alone to list other styles.
#[component]
pub fn DagLegend(styles: Vec<EdgeStyle>) -> impl IntoView {
    let items = styles
        .into_iter()
        .map(|s| {
            let hue = s.hue.name();
            let mut line = format!("cl-dag__edge cl-dag__hue--{hue}");
            if s.dashed {
                line.push_str(" cl-dag__edge--dashed");
            }
            view! {
                <span class="cl-dag-legend__item">
                    <svg class="cl-dag-legend__sample" width="36" height="12" aria-hidden="true">
                        <path class=line d="M 2 6 L 28 6" />
                        <path
                            class=format!("cl-dag__arrowhead cl-dag__hue--{hue}")
                            d="M 26 2 L 34 6 L 26 10 z"
                        />
                    </svg>
                    <span>{s.label}</span>
                </span>
            }
        })
        .collect_view();
    view! { <div class="cl-dag-legend">{items}</div> }
}

// ---------------------------------------------------------------------------
// The old API, kept for compatibility
// ---------------------------------------------------------------------------

/// A graph node for [`Graph`] (the old API). New code: [`DagNode`].
#[derive(Clone, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    /// A token colour (`token::OK`, ...). [`Graph`] maps it to the status
    /// class of the same hue; a colour that is not a token draws neutral.
    pub color: String,
    /// Optional sub-label (e.g. a kind: "source", "sink").
    pub sublabel: Option<String>,
}

impl GraphNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            color: token::ICE.into(),
            sublabel: None,
        }
    }
    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = c.into();
        self
    }
    pub fn sublabel(mut self, s: impl Into<String>) -> Self {
        self.sublabel = Some(s.into());
        self
    }
}

impl From<GraphNode> for DagNode {
    fn from(n: GraphNode) -> Self {
        DagNode {
            status: Hue::from_token(&n.color).unwrap_or_default(),
            sublabel: n.sublabel,
            ..DagNode::new(n.id, n.label)
        }
    }
}

/// A directed edge `from → to` for [`Graph`] (the old API). `active`
/// animates it. New code: [`DagEdge`].
#[derive(Clone, PartialEq)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub active: bool,
}

impl GraphEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            active: false,
        }
    }
    pub fn active(mut self, a: bool) -> Self {
        self.active = a;
        self
    }
}

impl From<GraphEdge> for DagEdge {
    fn from(e: GraphEdge) -> Self {
        DagEdge::new(e.from, e.to).style(if e.active { "active" } else { DEFAULT_STYLE })
    }
}

/// A computed node position (center point), from [`layout_dag`].
pub struct NodePos {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

/// The node centres and the canvas size, from the layout that [`Graph`]
/// uses. Kept for compatibility; new code calls [`graph_layout::layout`],
/// which also gives edge routes and lanes.
pub fn layout_dag(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    lr: bool,
    node_w: f64,
    node_h: f64,
) -> (Vec<NodePos>, f64, f64) {
    let dag_nodes: Vec<DagNode> = nodes.iter().cloned().map(DagNode::from).collect();
    let dag_edges: Vec<DagEdge> = edges.iter().cloned().map(DagEdge::from).collect();
    let opts = LayoutOptions {
        direction: if lr {
            Direction::LeftRight
        } else {
            Direction::TopBottom
        },
        node_w,
        node_h,
        ..LayoutOptions::default()
    };
    let l = graph_layout::layout(&dag_nodes, &dag_edges, &[], &[], &opts);
    let pos = l
        .nodes
        .iter()
        .map(|p| NodePos {
            id: p.id.clone(),
            x: p.cx(),
            y: p.cy(),
        })
        .collect();
    (pos, l.width, l.height)
}

/// The old graph component: `nodes` + `edges`, auto-laid-out. A thin wrapper
/// over [`Dag`]; new code uses [`Dag`] for lanes, fixed layers, edge styles
/// and a legend.
#[component]
pub fn Graph(
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    /// "TB" (top-to-bottom, default) or "LR" (left-to-right).
    #[prop(optional, into)]
    direction: String,
    #[prop(default = 150.0)] node_w: f64,
    #[prop(default = 48.0)] node_h: f64,
    /// Click, Space or Enter on a node.
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    /// Double click, or Enter on the selected node.
    #[prop(optional)]
    on_open: Option<Callback<String>>,
) -> impl IntoView {
    let direction = if direction.eq_ignore_ascii_case("LR") {
        Direction::LeftRight
    } else {
        Direction::TopBottom
    };
    let nodes: Vec<DagNode> = nodes.into_iter().map(DagNode::from).collect();
    let edges: Vec<DagEdge> = edges.into_iter().map(DagEdge::from).collect();
    let options = LayoutOptions {
        direction,
        node_w,
        node_h,
        ..LayoutOptions::default()
    };
    let select = Callback::new(move |id: String| {
        if let Some(cb) = on_select {
            cb.run(id);
        }
    });
    let open = Callback::new(move |id: String| {
        if let Some(cb) = on_open {
            cb.run(id);
        }
    });
    view! { <Dag nodes edges options on_select=select on_open=open /> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_adds_an_ellipsis_only_when_needed() {
        assert_eq!(clip("short", 100.0, 7.0), "short");
        assert_eq!(clip("a very long label", 42.0, 7.0), "a ver…");
    }

    #[test]
    fn built_in_styles_are_added_under_the_product_styles() {
        let table = style_table(Some(vec![EdgeStyle::new("default", "Mine", Hue::Ok)]));
        assert_eq!(table[0].label, "Mine", "the product wins");
        assert!(table.iter().any(|s| s.name == "active"));
    }

    #[test]
    fn only_used_styles_are_listed_and_unknown_names_draw_neutral() {
        let table = style_table(Some(vec![
            EdgeStyle::new("ok", "Succeeded", Hue::Ok),
            EdgeStyle::new("bad", "Failed", Hue::Bad),
        ]));
        let edges = vec![
            DagEdge::new("a", "b").style("bad"),
            DagEdge::new("b", "c").style("odd"),
        ];
        let used = used_styles(&table, &edges);
        let names: Vec<&str> = used.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["bad", "odd"]);
        assert_eq!(used[1].hue, Hue::Neutral);
    }

    #[test]
    fn the_old_types_convert() {
        let n: DagNode = GraphNode::new("a", "A")
            .color(token::BAD)
            .sublabel("s")
            .into();
        assert_eq!((n.status, n.sublabel.as_deref()), (Hue::Bad, Some("s")));
        let n: DagNode = GraphNode::new("a", "A").color("hotpink").into();
        assert_eq!(n.status, Hue::Neutral);
        let e: DagEdge = GraphEdge::new("a", "b").active(true).into();
        assert_eq!(e.style, "active");
        let e: DagEdge = GraphEdge::new("a", "b").into();
        assert_eq!(e.style, DEFAULT_STYLE);
    }

    #[test]
    fn layout_dag_still_gives_centres_and_a_size() {
        let nodes = vec![GraphNode::new("a", "A"), GraphNode::new("b", "B")];
        let edges = vec![GraphEdge::new("a", "b")];
        let (pos, w, h) = layout_dag(&nodes, &edges, true, 150.0, 48.0);
        assert_eq!(pos.len(), 2);
        assert!(pos[0].x < pos[1].x, "left to right");
        assert!(w > 300.0 && h > 48.0);
    }
}
