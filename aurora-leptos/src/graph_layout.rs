//! The data model and the layout of the Aurora graph ([`Dag`](crate::graph::Dag)).
//!
//! Pure code: no Leptos, no DOM. The same input gives the same geometry, so
//! positions do not jump when a product fetches the data again. Native unit
//! tests prove it.
//!
//! # Data model
//! A product gives:
//! - [`DagNode`]s: an id, a label, an optional sub-label and detail line, a
//!   status ([`Hue`]), flags (current, archived, done), marks, and a "+N" count.
//!   A node can have a fixed `layer`, and a `lane` (containment group).
//! - [`DagEdge`]s: `from`, `to`, and the name of an [`EdgeStyle`].
//! - Optional [`DagLane`]s: the label of a lane and its anchor node.
//! - Optional layer labels (column headers).
//!
//! # Two shapes
//! 1. **Ranked DAG.** Nodes have no `layer`. The layout computes the rank of
//!    each node from the edges (longest path from the sources). Cycles are
//!    broken for the ranking only; every edge is still drawn.
//! 2. **Fixed layers.** The product gives the `layer` of each node (for
//!    example Strategy | Initiative | Task). A `lane` groups the nodes of a
//!    layer in a band; the band follows its anchor node.
//!
//! The two can mix: a node with no `layer` is ranked after its fixed
//! predecessors.
//!
//! # Algorithm (a small Sugiyama)
//! 1. Rank (fixed or longest path over the edges, back edges reversed).
//! 2. Split each edge that spans more than one layer with dummy points, so it
//!    routes between the nodes, not through them.
//! 3. Order each layer: start from the sort key, then barycenter sweeps down
//!    and up. The members of a lane stay together. The order with the fewest
//!    crossings wins.
//! 4. Place each layer: stack the nodes ([`Align::Start`]), or also move them
//!    towards their neighbours ([`Align::Balanced`]) with no overlap.
//! 5. Route each edge as a smooth path from box face to box face, through
//!    its dummy points.
//!
//! Limits: one lane per node; one size for all nodes; self loops are not
//! drawn; the crossing reduction is a heuristic, not a minimum.

use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A token hue. It names a status of a node, the colour of an edge style, or
/// the colour of a mark. Each hue maps to the `--x` / `--x-fg` / `--x-bg`
/// tokens in `style/tokens.css`, through a CSS class. `Neutral` is the default
/// look (no status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Hue {
    #[default]
    Neutral,
    Ice,
    Teal,
    Violet,
    Gold,
    Ok,
    Bad,
    Skip,
    Muted,
}

impl Hue {
    /// All the hues, in a fixed order.
    pub const ALL: [Hue; 9] = [
        Hue::Neutral,
        Hue::Ice,
        Hue::Teal,
        Hue::Violet,
        Hue::Gold,
        Hue::Ok,
        Hue::Bad,
        Hue::Skip,
        Hue::Muted,
    ];

    /// The token name: `ok` for [`Hue::Ok`], `neutral` for [`Hue::Neutral`].
    pub fn name(self) -> &'static str {
        match self {
            Hue::Neutral => "neutral",
            Hue::Ice => "ice",
            Hue::Teal => "teal",
            Hue::Violet => "violet",
            Hue::Gold => "gold",
            Hue::Ok => "ok",
            Hue::Bad => "bad",
            Hue::Skip => "skip",
            Hue::Muted => "muted",
        }
    }

    /// The status class of a node: `status-ok`, `status-neutral`, ...
    pub fn status_class(self) -> &'static str {
        match self {
            Hue::Neutral => "status-neutral",
            Hue::Ice => "status-ice",
            Hue::Teal => "status-teal",
            Hue::Violet => "status-violet",
            Hue::Gold => "status-gold",
            Hue::Ok => "status-ok",
            Hue::Bad => "status-bad",
            Hue::Skip => "status-skip",
            Hue::Muted => "status-muted",
        }
    }

    /// The hue of a token string: `var(--ok)` (or `var(--ok-fg)`,
    /// `var(--ok-bg)`) gives [`Hue::Ok`]. `var(--faint)` gives
    /// [`Hue::Muted`]. Any other string gives `None`.
    pub fn from_token(color: &str) -> Option<Hue> {
        let name = color.trim().strip_prefix("var(--")?.strip_suffix(')')?;
        let name = name
            .strip_suffix("-fg")
            .or_else(|| name.strip_suffix("-bg"))
            .unwrap_or(name);
        Some(match name {
            "ice" => Hue::Ice,
            "teal" => Hue::Teal,
            "violet" => Hue::Violet,
            "gold" => Hue::Gold,
            "ok" => Hue::Ok,
            "bad" => Hue::Bad,
            "skip" => Hue::Skip,
            "muted" | "faint" => Hue::Muted,
            "edge" => Hue::Neutral,
            _ => return None,
        })
    }

    /// The usual hue of a status word (`completed` → [`Hue::Ok`]). The same
    /// map as [`status_color`](crate::tokens::status_color). Unknown words
    /// give [`Hue::Muted`].
    pub fn from_status(status: &str) -> Hue {
        match status.to_lowercase().as_str() {
            "running" => Hue::Ice,
            "completed" => Hue::Ok,
            "failed" => Hue::Bad,
            "scheduled" => Hue::Violet,
            "cancelled" | "canceled" => Hue::Gold,
            "skipped" => Hue::Skip,
            _ => Hue::Muted,
        }
    }
}

/// A short word at the bottom right of a node (for example "done" or
/// "put away"), in the `-fg` token of its hue.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMark {
    pub label: String,
    pub hue: Hue,
}

/// A node of the graph. Use [`DagNode::new`] and the builder methods.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DagNode {
    /// Unique id. The callbacks give it back. A second node with the same id
    /// is ignored.
    pub id: String,
    /// First line.
    pub label: String,
    /// Second line (small, mono).
    pub sublabel: Option<String>,
    /// Third line (small), for example a status word.
    pub detail: Option<String>,
    /// A free word for product CSS (`data-kind` on the node).
    pub kind: Option<String>,
    /// A fixed layer (column). `None`: the layout computes the rank.
    pub layer: Option<usize>,
    /// The id of the lane (containment group) of the node.
    pub lane: Option<String>,
    /// The status. It sets the `status-*` class.
    pub status: Hue,
    /// The node the view is about: a heavier border.
    pub current: bool,
    /// An archived node: drawn, but with a dashed border and an inset fill.
    pub archived: bool,
    /// A finished node: a class hook (`cl-dag__node--done`).
    pub done: bool,
    /// Short words at the bottom right.
    pub marks: Vec<NodeMark>,
    /// The number of neighbours that are not shown. More than 0 shows a
    /// "+N" badge.
    pub more: u32,
    /// The hover text. Default: the label, the sub-label and the detail.
    pub tooltip: Option<String>,
    /// The key that breaks ties in the order of a layer. Default: the id.
    pub sort_key: Option<String>,
}

impl DagNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            ..Self::default()
        }
    }
    pub fn sublabel(mut self, s: impl Into<String>) -> Self {
        self.sublabel = Some(s.into());
        self
    }
    pub fn detail(mut self, s: impl Into<String>) -> Self {
        self.detail = Some(s.into());
        self
    }
    pub fn kind(mut self, s: impl Into<String>) -> Self {
        self.kind = Some(s.into());
        self
    }
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = Some(layer);
        self
    }
    pub fn lane(mut self, lane: impl Into<String>) -> Self {
        self.lane = Some(lane.into());
        self
    }
    pub fn status(mut self, hue: Hue) -> Self {
        self.status = hue;
        self
    }
    pub fn current(mut self, on: bool) -> Self {
        self.current = on;
        self
    }
    pub fn archived(mut self, on: bool) -> Self {
        self.archived = on;
        self
    }
    pub fn done(mut self, on: bool) -> Self {
        self.done = on;
        self
    }
    pub fn mark(mut self, label: impl Into<String>, hue: Hue) -> Self {
        self.marks.push(NodeMark {
            label: label.into(),
            hue,
        });
        self
    }
    pub fn more(mut self, n: u32) -> Self {
        self.more = n;
        self
    }
    pub fn tooltip(mut self, s: impl Into<String>) -> Self {
        self.tooltip = Some(s.into());
        self
    }
    pub fn sort_key(mut self, s: impl Into<String>) -> Self {
        self.sort_key = Some(s.into());
        self
    }

    /// The key that orders the node in its layer.
    pub fn key(&self) -> &str {
        self.sort_key.as_deref().unwrap_or(&self.id)
    }
}

/// The name of the built-in default edge style.
pub const DEFAULT_STYLE: &str = "default";

/// A directed edge `from → to`, drawn in the named [`EdgeStyle`].
#[derive(Debug, Clone, PartialEq)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
    /// The name of an [`EdgeStyle`]. An unknown name draws in the neutral
    /// style.
    pub style: String,
    /// The hover text. Default: the label of the style.
    pub label: Option<String>,
}

impl DagEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            style: DEFAULT_STYLE.into(),
            label: None,
        }
    }
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = style.into();
        self
    }
    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = Some(s.into());
        self
    }
}

/// A lane: a band around the nodes of one layer that share a `lane` id.
/// Give one to set the label, or an anchor node that the band follows (for
/// example the parent of the nodes). A lane id with no `DagLane` gets a band
/// labelled with the id.
#[derive(Debug, Clone, PartialEq)]
pub struct DagLane {
    pub id: String,
    pub label: String,
    /// The node that the band follows in the order (a parent).
    pub anchor: Option<String>,
}

impl DagLane {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            anchor: None,
        }
    }
    pub fn anchor(mut self, node_id: impl Into<String>) -> Self {
        self.anchor = Some(node_id.into());
        self
    }
}

/// How an edge style looks. The product names the styles; each edge names
/// one.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeStyle {
    pub name: String,
    /// The text in the legend.
    pub label: String,
    pub hue: Hue,
    pub dashed: bool,
    /// A slow pulse (for example a running step).
    pub animated: bool,
}

impl EdgeStyle {
    pub fn new(name: impl Into<String>, label: impl Into<String>, hue: Hue) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            hue,
            dashed: false,
            animated: false,
        }
    }
    pub fn dashed(mut self) -> Self {
        self.dashed = true;
        self
    }
    pub fn animated(mut self) -> Self {
        self.animated = true;
        self
    }
}

/// The built-in styles: `default` (neutral) and `active` (ice, pulse).
pub fn default_edge_styles() -> Vec<EdgeStyle> {
    vec![
        EdgeStyle::new(DEFAULT_STYLE, "Dependency", Hue::Neutral),
        EdgeStyle::new("active", "Active", Hue::Ice).animated(),
    ]
}

/// The direction of the layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Layers are columns, left to right.
    #[default]
    LeftRight,
    /// Layers are rows, top to bottom.
    TopBottom,
}

impl Direction {
    /// `"TB"` (any case) is top to bottom; all else is left to right.
    pub fn parse(s: &str) -> Direction {
        if s.eq_ignore_ascii_case("TB") {
            Direction::TopBottom
        } else {
            Direction::LeftRight
        }
    }
}

/// How the nodes of a layer are placed along the layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Move nodes towards their neighbours (straight chains), no overlap.
    #[default]
    Balanced,
    /// Stack the nodes from the start of the layer (a fixed, list-like look).
    Start,
}

/// Sizes and gaps of the layout, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOptions {
    pub direction: Direction,
    pub align: Align,
    pub node_w: f64,
    pub node_h: f64,
    /// The gap between two layers.
    pub layer_gap: f64,
    /// The gap between two nodes of a layer.
    pub node_gap: f64,
    /// The space between a lane band and its nodes.
    pub lane_pad: f64,
    /// The extra gap between two groups when one is a lane.
    pub lane_gap: f64,
    pub margin: f64,
    /// Barycenter sweeps (down and up) for the order.
    pub sweeps: usize,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            direction: Direction::LeftRight,
            align: Align::Balanced,
            node_w: 168.0,
            node_h: 48.0,
            layer_gap: 72.0,
            node_gap: 20.0,
            lane_pad: 8.0,
            lane_gap: 10.0,
            margin: 16.0,
            sweeps: 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// A placed node box (top-left corner and size).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedNode {
    pub id: String,
    /// The index of the node in the input slice.
    pub index: usize,
    pub layer: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl PlacedNode {
    pub fn cx(&self) -> f64 {
        self.x + self.w / 2.0
    }
    pub fn cy(&self) -> f64 {
        self.y + self.h / 2.0
    }
}

/// A routed edge.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedEdge {
    /// The index of the edge in the input slice.
    pub index: usize,
    pub from: String,
    pub to: String,
    /// The route: box face, dummy points, box face.
    pub points: Vec<(f64, f64)>,
    /// The SVG path (`M ... C ...`) through the points.
    pub path: String,
}

/// A lane band in one layer.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLane {
    pub id: String,
    pub label: String,
    pub layer: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// Where the label goes (text baseline, start).
    pub label_x: f64,
    pub label_y: f64,
}

/// A layer header.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedHeader {
    pub label: String,
    pub layer: usize,
    pub x: f64,
    pub y: f64,
}

/// The finished geometry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub width: f64,
    pub height: f64,
    /// In reading order: by layer, then along the layer. This is also the
    /// keyboard order.
    pub nodes: Vec<PlacedNode>,
    /// In input order.
    pub edges: Vec<PlacedEdge>,
    pub lanes: Vec<PlacedLane>,
    pub headers: Vec<PlacedHeader>,
}

impl Layout {
    /// The placed node with this id.
    pub fn node(&self, id: &str) -> Option<&PlacedNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// Room for the layer headers: above the columns (left to right) or at the
/// left of the rows (top to bottom).
const HEADER_H: f64 = 28.0;
const HEADER_W: f64 = 96.0;
/// Room for a lane label above its band.
const LANE_LABEL: f64 = 14.0;
/// How far an edge between two nodes of one layer bows out.
const BOW: f64 = 44.0;
/// The gap between two dummy points.
const DUMMY_GAP: f64 = 10.0;

/// One thing that a layer holds: a node or a dummy point of a long edge.
struct Item {
    layer: usize,
    /// The node (index into the kept nodes), or `None` for a dummy.
    node: Option<usize>,
    lane: Option<usize>,
    /// Tie break: (lane-less, key rank, second key, edge).
    tie: (usize, usize, usize, usize),
}

/// A drawn edge after the split.
struct Chain {
    edge: usize,
    /// Items from the lower layer to the higher layer.
    items: Vec<usize>,
    /// The edge points from the higher layer to the lower one.
    reversed: bool,
    same_layer: bool,
}

fn cmp_f(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

fn num(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

/// Lay out the graph. Deterministic: the same input gives the same output,
/// and the order of the input nodes does not matter (ties break on
/// [`DagNode::key`]).
pub fn layout(
    nodes: &[DagNode],
    edges: &[DagEdge],
    lanes: &[DagLane],
    layers: &[String],
    opts: &LayoutOptions,
) -> Layout {
    let lr = opts.direction == Direction::LeftRight;

    // ---- index the nodes (the first node with an id wins) ------------------
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut kept: Vec<usize> = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        if !index.contains_key(node.id.as_str()) {
            index.insert(node.id.as_str(), kept.len());
            kept.push(i);
        }
    }
    let n = kept.len();
    let node = |k: usize| &nodes[kept[k]];

    let mut by_key: Vec<usize> = (0..n).collect();
    by_key.sort_by(|&a, &b| {
        node(a)
            .key()
            .cmp(node(b).key())
            .then(node(a).id.cmp(&node(b).id))
    });
    let mut key_rank = vec![0usize; n];
    for (rank, &k) in by_key.iter().enumerate() {
        key_rank[k] = rank;
    }

    // ---- lanes -------------------------------------------------------------
    // (id, label, anchor) — the given lanes first, then lane ids that only
    // the nodes name, in key order.
    let mut lane_table: Vec<(String, String, Option<usize>)> = Vec::new();
    let mut lane_index: HashMap<String, usize> = HashMap::new();
    for lane in lanes {
        if lane_index.contains_key(&lane.id) {
            continue;
        }
        lane_index.insert(lane.id.clone(), lane_table.len());
        let anchor = lane.anchor.as_deref().and_then(|a| index.get(a).copied());
        lane_table.push((lane.id.clone(), lane.label.clone(), anchor));
    }
    for &k in &by_key {
        if let Some(id) = &node(k).lane {
            if !lane_index.contains_key(id) {
                lane_index.insert(id.clone(), lane_table.len());
                lane_table.push((id.clone(), id.clone(), None));
            }
        }
    }
    let lane_of: Vec<Option<usize>> = (0..n)
        .map(|k| node(k).lane.as_ref().map(|id| lane_index[id]))
        .collect();

    // ---- drawn edges -------------------------------------------------------
    // (from, to, input edge index); unknown ends and self loops dropped.
    let links: Vec<(usize, usize, usize)> = edges
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let u = *index.get(e.from.as_str())?;
            let v = *index.get(e.to.as_str())?;
            (u != v).then_some((u, v, i))
        })
        .collect();

    // ---- 1. rank -----------------------------------------------------------
    let rank = rank_nodes(n, &links, &key_rank, &by_key, |k| node(k).layer);
    let layer_count = rank
        .iter()
        .map(|r| r + 1)
        .max()
        .unwrap_or(0)
        .max(layers.len())
        .max(1);

    // ---- 2. split long edges ----------------------------------------------
    let mut items: Vec<Item> = (0..n)
        .map(|k| Item {
            layer: rank[k],
            node: Some(k),
            lane: lane_of[k],
            tie: (usize::from(lane_of[k].is_none()), key_rank[k], 0, 0),
        })
        .collect();
    let mut chains: Vec<Chain> = Vec::with_capacity(links.len());
    // Segments between adjacent layers: (item in layer l, item in layer l+1).
    let mut segments: Vec<(usize, usize)> = Vec::new();
    for &(u, v, e) in &links {
        let (ru, rv) = (rank[u], rank[v]);
        if ru == rv {
            chains.push(Chain {
                edge: e,
                items: vec![u, v],
                reversed: false,
                same_layer: true,
            });
            continue;
        }
        let (a, b, reversed) = if ru < rv { (u, v, false) } else { (v, u, true) };
        let mut chain = vec![a];
        let mut prev = a;
        for layer in rank[a] + 1..rank[b] {
            let d = items.len();
            items.push(Item {
                layer,
                node: None,
                lane: None,
                tie: (1, key_rank[a], 1 + key_rank[b], e),
            });
            segments.push((prev, d));
            chain.push(d);
            prev = d;
        }
        segments.push((prev, b));
        chain.push(b);
        chains.push(Chain {
            edge: e,
            items: chain,
            reversed,
            same_layer: false,
        });
    }

    // Neighbours for the order: in lower layers (`up`) and higher (`down`).
    let mut up_nb: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    let mut down_nb: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    for &(p, q) in &segments {
        down_nb[p].push(q);
        up_nb[q].push(p);
    }
    // Containment pulls the members of a lane towards its anchor.
    for k in 0..n {
        if let Some(anchor) = lane_of[k].and_then(|l| lane_table[l].2) {
            if anchor == k {
                continue;
            }
            match rank[anchor].cmp(&rank[k]) {
                std::cmp::Ordering::Less => {
                    up_nb[k].push(anchor);
                    down_nb[anchor].push(k);
                }
                std::cmp::Ordering::Greater => {
                    down_nb[k].push(anchor);
                    up_nb[anchor].push(k);
                }
                std::cmp::Ordering::Equal => {}
            }
        }
    }

    // ---- 3. order ----------------------------------------------------------
    let mut order: Vec<Vec<usize>> = vec![Vec::new(); layer_count];
    for (i, item) in items.iter().enumerate() {
        order[item.layer].push(i);
    }
    let mut pos = vec![0.0f64; items.len()];
    let set_pos = |layer: &[usize], pos: &mut Vec<f64>| {
        let mid = (layer.len() as f64 - 1.0) / 2.0;
        for (at, &i) in layer.iter().enumerate() {
            pos[i] = at as f64 - mid;
        }
    };
    for layer in order.iter_mut() {
        layer.sort_by_key(|&i| items[i].tie);
        set_pos(layer, &mut pos);
        arrange(layer, &items, &pos, |_| None);
        set_pos(layer, &mut pos);
    }
    let mut seg_by_layer: Vec<Vec<(usize, usize)>> = vec![Vec::new(); layer_count];
    for &(p, q) in &segments {
        seg_by_layer[items[p].layer].push((p, q));
    }
    // A lane member and its anchor in the next layer count as a segment too:
    // a band that crosses the band of its neighbour reads as a crossing.
    for k in 0..n {
        if let Some(anchor) = lane_of[k].and_then(|l| lane_table[l].2) {
            if rank[anchor] + 1 == rank[k] {
                seg_by_layer[rank[anchor]].push((anchor, k));
            } else if rank[k] + 1 == rank[anchor] {
                seg_by_layer[rank[k]].push((k, anchor));
            }
        }
    }
    let mut best = order.clone();
    let mut best_crossings = crossings(&order, &seg_by_layer, items.len());
    for sweep in 0..opts.sweeps {
        if sweep > 0 && best_crossings == 0 {
            break;
        }
        for layer in order.iter_mut().skip(1) {
            let snapshot = pos.clone();
            arrange(layer, &items, &snapshot, |i| mean(&up_nb[i], &snapshot));
            set_pos(layer, &mut pos);
        }
        for layer in order.iter_mut().rev().skip(1) {
            let snapshot = pos.clone();
            arrange(layer, &items, &snapshot, |i| mean(&down_nb[i], &snapshot));
            set_pos(layer, &mut pos);
        }
        let c = crossings(&order, &seg_by_layer, items.len());
        // A tie takes the later order: it is closer to the barycenters.
        if c <= best_crossings {
            best_crossings = c;
            best = order.clone();
        }
    }
    let order = best;

    // ---- 4. place ----------------------------------------------------------
    let (across_size, along_size) = if lr {
        (opts.node_w, opts.node_h)
    } else {
        (opts.node_h, opts.node_w)
    };
    let label_room = if lr { LANE_LABEL } else { 0.0 };
    let size = |i: usize| {
        if items[i].node.is_some() {
            along_size
        } else {
            0.0
        }
    };
    let top_extra = |i: usize| {
        if items[i].lane.is_some() {
            opts.lane_pad + label_room
        } else {
            0.0
        }
    };
    let bottom_extra = |i: usize| {
        if items[i].lane.is_some() {
            opts.lane_pad
        } else {
            0.0
        }
    };
    // The distance between the centres of two neighbours in a layer.
    let sep = |p: usize, q: usize| {
        let gap = match (items[p].node.is_some(), items[q].node.is_some()) {
            (true, true) => opts.node_gap,
            (false, false) => DUMMY_GAP,
            _ => opts.node_gap / 2.0,
        };
        let mut s = (size(p) + size(q)) / 2.0 + gap;
        let (lp, lq) = (items[p].lane, items[q].lane);
        if lp != lq || lp.is_none() {
            // Different groups (two lane-less items are two groups).
            if lp.is_some() {
                s += opts.lane_pad;
            }
            if lq.is_some() {
                s += opts.lane_pad + label_room;
            }
            if lp.is_some() || lq.is_some() {
                s += opts.lane_gap;
            }
        }
        s
    };
    let header_room = if layers.is_empty() {
        0.0
    } else if lr {
        HEADER_H
    } else {
        HEADER_W
    };
    let start = opts.margin + header_room;
    let mut coord = vec![0.0f64; items.len()];
    for layer in &order {
        let mut c = 0.0;
        for (at, &i) in layer.iter().enumerate() {
            c = if at == 0 {
                start + top_extra(i) + size(i) / 2.0
            } else {
                c + sep(layer[at - 1], i)
            };
            coord[i] = c;
        }
    }
    if opts.align == Align::Balanced && layer_count > 1 {
        for pass in 0..4 {
            let down = pass % 2 == 0;
            let layer_ids: Vec<usize> = if down {
                (1..layer_count).collect()
            } else {
                (0..layer_count - 1).rev().collect()
            };
            for l in layer_ids {
                let layer = &order[l];
                if layer.is_empty() {
                    continue;
                }
                let desired: Vec<f64> = layer
                    .iter()
                    .map(|&i| {
                        let nb = if down { &up_nb[i] } else { &down_nb[i] };
                        mean(nb, &coord).unwrap_or(coord[i])
                    })
                    .collect();
                let seps: Vec<f64> = (1..layer.len())
                    .map(|at| sep(layer[at - 1], layer[at]))
                    .collect();
                for (&i, c) in layer.iter().zip(balance(&desired, &seps)) {
                    coord[i] = c;
                }
            }
        }
        // Move everything so that the top of the highest layer is at `start`.
        let top = order
            .iter()
            .filter_map(|layer| layer.first())
            .map(|&i| coord[i] - size(i) / 2.0 - top_extra(i))
            .fold(f64::INFINITY, f64::min);
        if top.is_finite() {
            let shift = start - top;
            for c in coord.iter_mut() {
                *c += shift;
            }
        }
    }

    let across_start = opts.margin
        + if !lr && lane_of.iter().any(Option::is_some) {
            LANE_LABEL
        } else {
            0.0
        };
    let across_c = |layer: usize| {
        across_start + layer as f64 * (across_size + opts.layer_gap) + across_size / 2.0
    };
    let xy = |across: f64, along: f64| if lr { (across, along) } else { (along, across) };

    // Nodes, in reading order.
    let mut placed_nodes = Vec::with_capacity(n);
    for (l, layer) in order.iter().enumerate() {
        for &i in layer {
            if let Some(k) = items[i].node {
                let (cx, cy) = xy(across_c(l), coord[i]);
                placed_nodes.push(PlacedNode {
                    id: node(k).id.clone(),
                    index: kept[k],
                    layer: l,
                    x: cx - opts.node_w / 2.0,
                    y: cy - opts.node_h / 2.0,
                    w: opts.node_w,
                    h: opts.node_h,
                });
            }
        }
    }

    // Lane bands: one per run of lane members in a layer.
    let mut placed_lanes = Vec::new();
    for (l, layer) in order.iter().enumerate() {
        let mut at = 0;
        while at < layer.len() {
            let Some(lane) = items[layer[at]].lane else {
                at += 1;
                continue;
            };
            let first = layer[at];
            let mut last = first;
            while at + 1 < layer.len() && items[layer[at + 1]].lane == Some(lane) {
                at += 1;
                last = layer[at];
            }
            at += 1;
            let along0 = coord[first] - size(first) / 2.0 - opts.lane_pad;
            let along1 = coord[last] + size(last) / 2.0 + opts.lane_pad;
            let across0 = across_c(l) - across_size / 2.0 - opts.lane_pad;
            let across1 = across_c(l) + across_size / 2.0 + opts.lane_pad;
            let (x0, y0) = xy(across0, along0);
            let (x1, y1) = xy(across1, along1);
            placed_lanes.push(PlacedLane {
                id: lane_table[lane].0.clone(),
                label: lane_table[lane].1.clone(),
                layer: l,
                x: x0,
                y: y0,
                w: x1 - x0,
                h: y1 - y0,
                label_x: x0 + 6.0,
                label_y: y0 - 4.0,
            });
        }
    }

    // ---- 5. route ----------------------------------------------------------
    let half = across_size / 2.0;
    let mut any_same_layer = false;
    let mut placed_edges: Vec<PlacedEdge> = Vec::with_capacity(chains.len());
    for chain in &chains {
        let e = &edges[chain.edge];
        // Points as (across, along).
        let mut pts: Vec<(f64, f64)> = Vec::with_capacity(chain.items.len());
        let path;
        if chain.same_layer {
            any_same_layer = true;
            let (u, v) = (chain.items[0], chain.items[1]);
            let a = across_c(items[u].layer) + half;
            pts.push((a, coord[u]));
            pts.push((a, coord[v]));
            let (p0, p1) = (pts[0], pts[1]);
            path = svg_path(&[p0, (a + BOW, p0.1), (a + BOW, p1.1), p1], true, &xy);
        } else {
            let last = chain.items.len() - 1;
            for (at, &i) in chain.items.iter().enumerate() {
                let c = across_c(items[i].layer);
                let a = if at == 0 {
                    c + half
                } else if at == last {
                    c - half
                } else {
                    c
                };
                pts.push((a, coord[i]));
            }
            if chain.reversed {
                // The source is in the higher layer: the route leaves its
                // low face and enters the high face of the target.
                pts.reverse();
            }
            path = svg_path(&pts, false, &xy);
        }
        placed_edges.push(PlacedEdge {
            index: chain.edge,
            from: e.from.clone(),
            to: e.to.clone(),
            points: pts.iter().map(|&(a, b)| xy(a, b)).collect(),
            path,
        });
    }
    placed_edges.sort_by_key(|e| e.index);

    // ---- headers and size --------------------------------------------------
    let headers = layers
        .iter()
        .enumerate()
        .map(|(l, label)| {
            let (x, y) = if lr {
                (across_c(l) - half, opts.margin + 14.0)
            } else {
                (opts.margin, across_c(l))
            };
            PlacedHeader {
                label: label.clone(),
                layer: l,
                x,
                y,
            }
        })
        .collect();
    let across_end =
        across_c(layer_count - 1) + half + opts.margin + if any_same_layer { BOW } else { 0.0 };
    let along_end = order
        .iter()
        .filter_map(|layer| layer.last())
        .map(|&i| coord[i] + size(i) / 2.0 + bottom_extra(i))
        .fold(start, f64::max)
        + opts.margin;
    let (width, height) = xy(across_end, along_end);

    Layout {
        width,
        height,
        nodes: placed_nodes,
        edges: placed_edges,
        lanes: placed_lanes,
        headers,
    }
}

/// The rank of each node: its fixed layer, or the longest path from a
/// source over the edges. A depth-first search in key order finds the back
/// edges of cycles; they count reversed.
fn rank_nodes(
    n: usize,
    links: &[(usize, usize, usize)],
    key_rank: &[usize],
    by_key: &[usize],
    fixed: impl Fn(usize) -> Option<usize>,
) -> Vec<usize> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (li, &(u, _, _)) in links.iter().enumerate() {
        out[u].push(li);
    }
    for list in out.iter_mut() {
        list.sort_by_key(|&li| (key_rank[links[li].1], li));
    }
    // 0 = new, 1 = on the stack, 2 = done.
    let mut state = vec![0u8; n];
    let mut back = vec![false; links.len()];
    for &root in by_key {
        if state[root] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        state[root] = 1;
        while let Some(top) = stack.last_mut() {
            let (v, cursor) = *top;
            if cursor < out[v].len() {
                top.1 += 1;
                let li = out[v][cursor];
                let w = links[li].1;
                match state[w] {
                    0 => {
                        state[w] = 1;
                        stack.push((w, 0));
                    }
                    1 => back[li] = true,
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    // Longest path over the acyclic orientation (Kahn).
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indegree = vec![0usize; n];
    for (li, &(u, v, _)) in links.iter().enumerate() {
        let (a, b) = if back[li] { (v, u) } else { (u, v) };
        succ[a].push(b);
        indegree[b] += 1;
    }
    let mut rank: Vec<usize> = (0..n).map(|k| fixed(k).unwrap_or(0)).collect();
    let mut queue: VecDeque<usize> = by_key
        .iter()
        .copied()
        .filter(|&k| indegree[k] == 0)
        .collect();
    while let Some(a) = queue.pop_front() {
        for &b in &succ[a] {
            if fixed(b).is_none() && rank[b] < rank[a] + 1 {
                rank[b] = rank[a] + 1;
            }
            indegree[b] -= 1;
            if indegree[b] == 0 {
                queue.push_back(b);
            }
        }
    }
    rank
}

/// The mean position of `of`, or `None` when it is empty.
fn mean(of: &[usize], pos: &[f64]) -> Option<f64> {
    (!of.is_empty()).then(|| of.iter().map(|&i| pos[i]).sum::<f64>() / of.len() as f64)
}

/// Sort one layer by barycenter. An item with no barycenter keeps its
/// position. The members of a lane stay together: groups sort by the mean
/// barycenter of their members, members sort inside their group.
fn arrange(layer: &mut [usize], items: &[Item], pos: &[f64], bary: impl Fn(usize) -> Option<f64>) {
    if layer.len() < 2 {
        return;
    }
    let value: HashMap<usize, f64> = layer
        .iter()
        .map(|&i| (i, bary(i).unwrap_or(pos[i])))
        .collect();
    // Group key: a lane, or the item alone.
    let mut groups: Vec<(Option<usize>, Vec<usize>)> = Vec::new();
    let mut group_of_lane: HashMap<usize, usize> = HashMap::new();
    for &i in layer.iter() {
        match items[i].lane {
            Some(lane) => {
                if let Some(&g) = group_of_lane.get(&lane) {
                    groups[g].1.push(i);
                } else {
                    group_of_lane.insert(lane, groups.len());
                    groups.push((Some(lane), vec![i]));
                }
            }
            None => groups.push((None, vec![i])),
        }
    }
    for (_, members) in groups.iter_mut() {
        members.sort_by(|&a, &b| cmp_f(value[&a], value[&b]).then(items[a].tie.cmp(&items[b].tie)));
    }
    let group_value =
        |members: &[usize]| members.iter().map(|i| value[i]).sum::<f64>() / members.len() as f64;
    groups.sort_by(|(_, a), (_, b)| {
        cmp_f(group_value(a), group_value(b)).then(items[a[0]].tie.cmp(&items[b[0]].tie))
    });
    let mut at = 0;
    for (_, members) in groups {
        for i in members {
            layer[at] = i;
            at += 1;
        }
    }
}

/// The number of crossings between adjacent layers.
fn crossings(order: &[Vec<usize>], seg_by_layer: &[Vec<(usize, usize)>], len: usize) -> usize {
    let mut at = vec![0usize; len];
    for layer in order {
        for (i, &item) in layer.iter().enumerate() {
            at[item] = i;
        }
    }
    let mut total = 0;
    for segs in seg_by_layer {
        let ends: Vec<(usize, usize)> = segs.iter().map(|&(p, q)| (at[p], at[q])).collect();
        for i in 0..ends.len() {
            for j in i + 1..ends.len() {
                let (a, b) = (ends[i], ends[j]);
                if (a.0 < b.0 && a.1 > b.1) || (a.0 > b.0 && a.1 < b.1) {
                    total += 1;
                }
            }
        }
    }
    total
}

/// Move a layer towards the desired centres with no overlap: `seps[i]` is
/// the least distance between item `i` and item `i + 1`. The mean of a
/// push-down pass and a push-up pass meets every limit and has no bias.
fn balance(desired: &[f64], seps: &[f64]) -> Vec<f64> {
    let len = desired.len();
    let mut down = desired.to_vec();
    for i in 1..len {
        down[i] = down[i].max(down[i - 1] + seps[i - 1]);
    }
    let mut up = desired.to_vec();
    for i in (0..len.saturating_sub(1)).rev() {
        up[i] = up[i].min(up[i + 1] - seps[i]);
    }
    down.iter().zip(&up).map(|(d, u)| (d + u) / 2.0).collect()
}

/// An SVG path through points given as (across, along). Each step is a
/// cubic curve that leaves and enters along the across axis. `bow`: the
/// four points are start, two controls, end.
fn svg_path(pts: &[(f64, f64)], bow: bool, xy: &impl Fn(f64, f64) -> (f64, f64)) -> String {
    let p = |a: f64, b: f64| {
        let (x, y) = xy(a, b);
        format!("{} {}", num(x), num(y))
    };
    let mut d = format!("M {}", p(pts[0].0, pts[0].1));
    if bow {
        d.push_str(&format!(
            " C {}, {}, {}",
            p(pts[1].0, pts[1].1),
            p(pts[2].0, pts[2].1),
            p(pts[3].0, pts[3].1)
        ));
        return d;
    }
    for w in pts.windows(2) {
        let (a0, b0) = w[0];
        let (a1, b1) = w[1];
        let k = (a1 - a0) / 2.0;
        d.push_str(&format!(
            " C {}, {}, {}",
            p(a0 + k, b0),
            p(a1 - k, b1),
            p(a1, b1)
        ));
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn opts() -> LayoutOptions {
        LayoutOptions::default()
    }

    fn run(nodes: &[DagNode], edges: &[DagEdge]) -> Layout {
        layout(nodes, edges, &[], &[], &opts())
    }

    fn placed<'a>(l: &'a Layout, id: &str) -> &'a PlacedNode {
        l.node(id).unwrap_or_else(|| panic!("{id} is not placed"))
    }

    // ---- ranking -----------------------------------------------------------

    #[test]
    fn ranks_follow_the_longest_path() {
        let nodes: Vec<DagNode> = ["a", "b", "c", "d"]
            .iter()
            .map(|i| DagNode::new(*i, *i))
            .collect();
        // a → b → c, and a → c: c is at rank 2, not 1.
        let edges = vec![
            DagEdge::new("a", "b"),
            DagEdge::new("b", "c"),
            DagEdge::new("a", "c"),
        ];
        let l = run(&nodes, &edges);
        assert_eq!(placed(&l, "a").layer, 0);
        assert_eq!(placed(&l, "b").layer, 1);
        assert_eq!(placed(&l, "c").layer, 2);
        // An isolated node is at rank 0.
        assert_eq!(placed(&l, "d").layer, 0);
        // Left to right: the columns go right.
        assert!(placed(&l, "a").x < placed(&l, "b").x);
        assert!(placed(&l, "b").x < placed(&l, "c").x);
    }

    #[test]
    fn a_cycle_does_not_hang_and_every_edge_is_drawn() {
        let nodes: Vec<DagNode> = ["a", "b", "c"]
            .iter()
            .map(|i| DagNode::new(*i, *i))
            .collect();
        let edges = vec![
            DagEdge::new("a", "b"),
            DagEdge::new("b", "c"),
            DagEdge::new("c", "a"),
        ];
        let l = run(&nodes, &edges);
        assert_eq!(l.edges.len(), 3);
        let layers: HashSet<usize> = l.nodes.iter().map(|n| n.layer).collect();
        assert_eq!(layers.len(), 3, "the cycle is broken at one edge");
    }

    #[test]
    fn fixed_layers_win_and_free_nodes_rank_after_them() {
        let nodes = vec![
            DagNode::new("s", "s").layer(0),
            DagNode::new("t", "t").layer(2),
            DagNode::new("free", "free"),
        ];
        let edges = vec![DagEdge::new("t", "free"), DagEdge::new("s", "t")];
        let l = run(&nodes, &edges);
        assert_eq!(placed(&l, "s").layer, 0);
        assert_eq!(placed(&l, "t").layer, 2);
        assert_eq!(placed(&l, "free").layer, 3);
    }

    #[test]
    fn unknown_ends_self_loops_and_duplicate_ids_are_ignored() {
        let nodes = vec![
            DagNode::new("a", "a"),
            DagNode::new("a", "again"),
            DagNode::new("b", "b"),
        ];
        let edges = vec![
            DagEdge::new("a", "ghost"),
            DagEdge::new("a", "a"),
            DagEdge::new("a", "b"),
        ];
        let l = run(&nodes, &edges);
        assert_eq!(l.nodes.len(), 2);
        assert_eq!(placed(&l, "a").index, 0, "the first node with an id wins");
        assert_eq!(l.edges.len(), 1);
        assert_eq!(l.edges[0].index, 2);
    }

    #[test]
    fn an_empty_graph_has_a_size() {
        let l = run(&[], &[]);
        assert!(l.nodes.is_empty());
        assert!(l.width > 0.0 && l.height > 0.0);
    }

    // ---- ordering ----------------------------------------------------------

    /// Two parallel chains given crossed: the key order puts x1 above y1 in
    /// layer 0 but y2 above x2 in layer 1 (keys "a…" sort first). The
    /// barycenter pass removes the crossing.
    #[test]
    fn barycenter_removes_a_crossing() {
        let nodes = vec![
            DagNode::new("x1", "x1").sort_key("a"),
            DagNode::new("y1", "y1").sort_key("b"),
            DagNode::new("x2", "x2").sort_key("d"),
            DagNode::new("y2", "y2").sort_key("c"),
        ];
        let edges = vec![DagEdge::new("x1", "x2"), DagEdge::new("y1", "y2")];
        let l = run(&nodes, &edges);
        let above = |a: &str, b: &str| placed(&l, a).y < placed(&l, b).y;
        assert_eq!(above("x1", "y1"), above("x2", "y2"), "no crossing");
    }

    #[test]
    fn crossings_counts_inversions() {
        // Layer 0: items 0, 1. Layer 1: items 2, 3. Edges 0→3 and 1→2 cross.
        let order = vec![vec![0, 1], vec![2, 3]];
        let segs = vec![vec![(0, 3), (1, 2)], vec![]];
        assert_eq!(crossings(&order, &segs, 4), 1);
        let order = vec![vec![0, 1], vec![3, 2]];
        assert_eq!(crossings(&order, &segs, 4), 0);
    }

    #[test]
    fn a_long_edge_routes_around_the_middle_node() {
        // a → b → c and a → c: the a → c edge gets a dummy point in layer 1,
        // not on top of b.
        let nodes: Vec<DagNode> = ["a", "b", "c"]
            .iter()
            .map(|i| DagNode::new(*i, *i))
            .collect();
        let edges = vec![
            DagEdge::new("a", "b"),
            DagEdge::new("b", "c"),
            DagEdge::new("a", "c"),
        ];
        let l = run(&nodes, &edges);
        let long = l
            .edges
            .iter()
            .find(|e| e.from == "a" && e.to == "c")
            .unwrap();
        assert_eq!(long.points.len(), 3, "start, one dummy, end");
        let b = placed(&l, "b");
        let (_, dummy_y) = long.points[1];
        assert!(
            dummy_y < b.y || dummy_y > b.y + b.h,
            "the dummy point is not inside b: {dummy_y} vs {}..{}",
            b.y,
            b.y + b.h
        );
    }

    // ---- stable output -----------------------------------------------------

    fn workflow() -> (Vec<DagNode>, Vec<DagEdge>) {
        let nodes = [
            "extract", "clean", "enrich", "score", "load", "report", "audit",
        ]
        .iter()
        .map(|i| DagNode::new(*i, *i))
        .collect();
        let edges = vec![
            DagEdge::new("extract", "clean"),
            DagEdge::new("extract", "enrich"),
            DagEdge::new("clean", "score"),
            DagEdge::new("enrich", "score"),
            DagEdge::new("score", "load"),
            DagEdge::new("load", "report"),
            DagEdge::new("extract", "audit"),
            DagEdge::new("audit", "report").style("bad"),
        ];
        (nodes, edges)
    }

    #[test]
    fn the_same_input_gives_the_same_layout() {
        let (nodes, edges) = workflow();
        assert_eq!(run(&nodes, &edges), run(&nodes, &edges));
    }

    #[test]
    fn the_input_order_does_not_change_the_geometry() {
        let (nodes, edges) = workflow();
        let first = run(&nodes, &edges);
        let mut reversed = nodes.clone();
        reversed.reverse();
        let second = run(&reversed, &edges);
        for node in &first.nodes {
            let other = placed(&second, &node.id);
            assert_eq!(
                (node.x, node.y, node.layer),
                (other.x, other.y, other.layer),
                "{}",
                node.id
            );
        }
        assert_eq!(
            first.edges.iter().map(|e| &e.path).collect::<Vec<_>>(),
            second.edges.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
    }

    #[test]
    fn nodes_do_not_overlap() {
        let (nodes, edges) = workflow();
        for direction in [Direction::LeftRight, Direction::TopBottom] {
            for align in [Align::Balanced, Align::Start] {
                let o = LayoutOptions {
                    direction,
                    align,
                    ..opts()
                };
                let l = layout(&nodes, &edges, &[], &[], &o);
                for a in &l.nodes {
                    assert!(a.x >= 0.0 && a.y >= 0.0);
                    assert!(a.x + a.w <= l.width && a.y + a.h <= l.height);
                    for b in &l.nodes {
                        if a.id < b.id {
                            let apart = a.x + a.w <= b.x
                                || b.x + b.w <= a.x
                                || a.y + a.h <= b.y
                                || b.y + b.h <= a.y;
                            assert!(
                                apart,
                                "{} and {} overlap ({direction:?}, {align:?})",
                                a.id, b.id
                            );
                        }
                    }
                }
            }
        }
    }

    // ---- edge routing ------------------------------------------------------

    #[test]
    fn an_edge_goes_from_face_to_face() {
        let nodes = vec![DagNode::new("a", "a"), DagNode::new("b", "b")];
        let edges = vec![DagEdge::new("a", "b")];
        let l = run(&nodes, &edges);
        let (a, b) = (placed(&l, "a"), placed(&l, "b"));
        let e = &l.edges[0];
        assert_eq!(e.points, vec![(a.x + a.w, a.cy()), (b.x, b.cy())]);
        assert!(e.path.starts_with("M "));
        assert_eq!(e.path.matches(" C ").count(), 1);

        // Top to bottom: bottom face to top face.
        let o = LayoutOptions {
            direction: Direction::TopBottom,
            ..opts()
        };
        let l = layout(&nodes, &edges, &[], &[], &o);
        let (a, b) = (placed(&l, "a"), placed(&l, "b"));
        assert_eq!(l.edges[0].points, vec![(a.cx(), a.y + a.h), (b.cx(), b.y)]);
    }

    #[test]
    fn a_backward_edge_leaves_the_left_face() {
        // Fixed layers make b → a point backwards.
        let nodes = vec![
            DagNode::new("a", "a").layer(0),
            DagNode::new("b", "b").layer(1),
        ];
        let edges = vec![DagEdge::new("b", "a")];
        let l = run(&nodes, &edges);
        let (a, b) = (placed(&l, "a"), placed(&l, "b"));
        assert_eq!(l.edges[0].points, vec![(b.x, b.cy()), (a.x + a.w, a.cy())]);
    }

    #[test]
    fn an_edge_in_one_layer_bows_out() {
        let nodes = vec![
            DagNode::new("a", "a").layer(0),
            DagNode::new("b", "b").layer(0),
        ];
        let edges = vec![DagEdge::new("a", "b")];
        let l = run(&nodes, &edges);
        let (a, b) = (placed(&l, "a"), placed(&l, "b"));
        assert_eq!(
            l.edges[0].points,
            vec![(a.x + a.w, a.cy()), (b.x + b.w, b.cy())]
        );
        assert!(l.width >= a.x + a.w + BOW, "room for the bow");
    }

    // ---- lanes (ported from Kairos graph_layout.rs) ------------------------

    /// The Kairos flight-level fixture: Strategy | Initiative | Task as fixed
    /// layers; `parent` is containment (lanes), `blocks` the only edge.
    fn flight_levels() -> (Vec<DagNode>, Vec<DagEdge>, Vec<DagLane>, Vec<String>) {
        let nodes = vec![
            DagNode::new("s1", "A-S-0001").layer(0).sort_key("A-S-0001"),
            DagNode::new("i1", "A-I-0001")
                .layer(1)
                .lane("s1")
                .sort_key("A-I-0001"),
            DagNode::new("i2", "A-I-0002")
                .layer(1)
                .lane("s1")
                .sort_key("A-I-0002"),
            DagNode::new("t1", "A-T-0001")
                .layer(2)
                .lane("i1")
                .sort_key("A-T-0001"),
            DagNode::new("t2", "A-T-0002")
                .layer(2)
                .lane("i1")
                .sort_key("A-T-0002"),
            DagNode::new("t3", "A-T-0003")
                .layer(2)
                .lane("i2")
                .sort_key("A-T-0003")
                .more(1),
        ];
        let edges = vec![DagEdge::new("t1", "t3").style("open")];
        let lanes = vec![
            DagLane::new("s1", "A-S-0001").anchor("s1"),
            DagLane::new("i1", "A-I-0001").anchor("i1"),
            DagLane::new("i2", "A-I-0002").anchor("i2"),
        ];
        let layers = vec!["Strategy".into(), "Initiative".into(), "Task".into()];
        (nodes, edges, lanes, layers)
    }

    fn kairos_opts() -> LayoutOptions {
        LayoutOptions {
            align: Align::Start,
            node_w: 220.0,
            node_h: 56.0,
            ..opts()
        }
    }

    #[test]
    fn kairos_layout_is_deterministic() {
        let (nodes, edges, lanes, layers) = flight_levels();
        let first = layout(&nodes, &edges, &lanes, &layers, &kairos_opts());
        let second = layout(&nodes, &edges, &lanes, &layers, &kairos_opts());
        assert_eq!(first, second);
        // Shuffled input changes nothing: ordering is by key and barycenter,
        // never by arrival order.
        let mut reversed = nodes.clone();
        reversed.reverse();
        let third = layout(&reversed, &edges, &lanes, &layers, &kairos_opts());
        let strip = |mut l: Layout| {
            for n in l.nodes.iter_mut() {
                n.index = 0;
            }
            l
        };
        assert_eq!(strip(first), strip(third));
    }

    #[test]
    fn kairos_columns_lanes_and_arrows() {
        let (nodes, edges, lanes, layers) = flight_levels();
        let result = layout(&nodes, &edges, &lanes, &layers, &kairos_opts());
        let p = |id: &str| placed(&result, id);
        // Fixed columns, left to right.
        assert!(p("s1").x < p("i1").x);
        assert!(p("i1").x < p("t1").x);
        assert_eq!(p("i1").x, p("i2").x);
        // t1/t2 band under i1, together and above the group of i2.
        assert!(p("t1").y < p("t2").y);
        assert!(p("t2").y < p("t3").y);
        // Lanes exist for both initiatives and the strategy.
        let lane_ids: Vec<&str> = result.lanes.iter().map(|l| l.id.as_str()).collect();
        assert!(lane_ids.contains(&"i1"));
        assert!(lane_ids.contains(&"i2"));
        assert!(lane_ids.contains(&"s1"));
        // The i1 lane spans t1 and t2, and holds them fully.
        let lane = result.lanes.iter().find(|l| l.id == "i1").unwrap();
        assert!(lane.y <= p("t1").y && lane.y + lane.h >= p("t2").y + p("t2").h);
        assert!(lane.x < p("t1").x && lane.x + lane.w > p("t1").x + p("t1").w);
        // The label sits above the band.
        assert!(lane.label_y < lane.y);
        // Exactly one edge: the blocks edge. Containment draws none.
        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.edges[0].from, "t1");
        // Three headers, one per column, at the column's left edge.
        assert_eq!(result.headers.len(), 3);
        assert_eq!(result.headers[2].x, p("t1").x);
        // The headers are above every node.
        assert!(result.nodes.iter().all(|n| n.y > result.headers[0].y));
    }

    #[test]
    fn lane_bands_do_not_touch() {
        let (nodes, edges, lanes, layers) = flight_levels();
        let result = layout(&nodes, &edges, &lanes, &layers, &kairos_opts());
        let a = result.lanes.iter().find(|l| l.id == "i1").unwrap();
        let b = result.lanes.iter().find(|l| l.id == "i2").unwrap();
        assert!(
            a.y + a.h < b.label_y - 8.0,
            "the i2 label has room above its band"
        );
    }

    #[test]
    fn a_lane_follows_its_anchor() {
        // Two parents; the lanes of the children follow their order, even
        // when the children's keys say the opposite.
        let nodes = vec![
            DagNode::new("p1", "p1").layer(0).sort_key("1"),
            DagNode::new("p2", "p2").layer(0).sort_key("2"),
            DagNode::new("c-of-2", "c")
                .layer(1)
                .lane("L2")
                .sort_key("a"),
            DagNode::new("c-of-1", "c")
                .layer(1)
                .lane("L1")
                .sort_key("b"),
        ];
        let lanes = vec![
            DagLane::new("L1", "p1").anchor("p1"),
            DagLane::new("L2", "p2").anchor("p2"),
        ];
        let l = layout(&nodes, &[], &lanes, &[], &kairos_opts());
        assert!(placed(&l, "c-of-1").y < placed(&l, "c-of-2").y);
    }

    #[test]
    fn a_lane_with_no_definition_is_labelled_with_its_id() {
        let nodes = vec![DagNode::new("a", "a").lane("group-x")];
        let l = run(&nodes, &[]);
        assert_eq!(l.lanes.len(), 1);
        assert_eq!(l.lanes[0].label, "group-x");
    }

    #[test]
    fn top_to_bottom_lanes_and_headers() {
        let (nodes, edges, lanes, layers) = flight_levels();
        let o = LayoutOptions {
            direction: Direction::TopBottom,
            ..kairos_opts()
        };
        let result = layout(&nodes, &edges, &lanes, &layers, &o);
        let p = |id: &str| placed(&result, id);
        assert!(p("s1").y < p("i1").y && p("i1").y < p("t1").y);
        assert!(p("t1").x < p("t2").x);
        let lane = result.lanes.iter().find(|l| l.id == "i1").unwrap();
        assert!(lane.x <= p("t1").x && lane.x + lane.w >= p("t2").x + p("t2").w);
        assert!(lane.label_y >= 0.0);
        assert!(result
            .nodes
            .iter()
            .all(|n| n.x > result.headers[0].x + 80.0));
    }

    // ---- model -------------------------------------------------------------

    #[test]
    fn hues_map_to_tokens_and_classes() {
        assert_eq!(Hue::from_token("var(--ok)"), Some(Hue::Ok));
        assert_eq!(Hue::from_token("var(--bad-fg)"), Some(Hue::Bad));
        assert_eq!(Hue::from_token("var(--faint)"), Some(Hue::Muted));
        assert_eq!(Hue::from_token("red"), None);
        assert_eq!(Hue::Ok.status_class(), "status-ok");
        assert_eq!(Hue::from_status("FAILED"), Hue::Bad);
        for hue in Hue::ALL {
            assert_eq!(hue.status_class(), format!("status-{}", hue.name()));
        }
    }

    // ---- performance -------------------------------------------------------

    /// 200 nodes and 400 edges in a layered random DAG. Prints the time; the
    /// bound is loose so that a slow CI machine does not fail it.
    #[test]
    fn layout_of_200_nodes_and_400_edges_is_fast() {
        let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let nodes: Vec<DagNode> = (0..200)
            .map(|i| DagNode::new(format!("n{i:03}"), format!("node {i}")))
            .collect();
        let mut edges = Vec::new();
        while edges.len() < 400 {
            let a = (next() % 200) as usize;
            let b = (next() % 200) as usize;
            if a < b && b - a < 40 {
                edges.push(DagEdge::new(format!("n{a:03}"), format!("n{b:03}")));
            }
        }
        let t = std::time::Instant::now();
        let l = run(&nodes, &edges);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        println!("layout of 200 nodes / 400 edges: {ms:.1} ms");
        assert_eq!(l.nodes.len(), 200);
        assert_eq!(l.edges.len(), 400);
        assert!(ms < 5000.0, "{ms} ms");
    }
}
