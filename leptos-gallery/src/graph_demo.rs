//! Graph demos: a workflow DAG (like Cloacina) and a layered graph with lanes
//! (like the Kairos flight levels). The gallery plays the product: it gives
//! the data, and shows what the callbacks give back.

use aurora_leptos::graph::*;
use leptos::prelude::*;

/// The id in a signal, or "none".
fn shown(s: RwSignal<Option<String>>) -> impl Fn() -> String {
    move || s.get().unwrap_or_else(|| "none".into())
}

/// A ranked DAG: ranks come from the edges, left to right.
#[component]
pub fn WorkflowDemo() -> impl IntoView {
    let selected = RwSignal::new(None::<String>);
    let opened = RwSignal::new(None::<String>);

    let task = |id: &str, status: &str| {
        DagNode::new(id, id)
            .sublabel("task")
            .detail(status)
            .status(Hue::from_status(status))
    };
    let nodes = vec![
        task("fetch_orders", "completed"),
        task("fetch_customers", "completed"),
        task("validate", "completed"),
        task("enrich", "running"),
        task("dedupe", "completed"),
        task("score", "pending"),
        task("load_warehouse", "pending"),
        task("audit_log", "failed"),
        task("notify", "skipped"),
    ];
    let edges = vec![
        DagEdge::new("fetch_orders", "validate").style("ok"),
        DagEdge::new("fetch_customers", "validate").style("ok"),
        DagEdge::new("validate", "enrich").style("active"),
        DagEdge::new("validate", "dedupe").style("ok"),
        DagEdge::new("dedupe", "score").style("waiting"),
        DagEdge::new("enrich", "score").style("waiting"),
        DagEdge::new("score", "load_warehouse").style("waiting"),
        DagEdge::new("fetch_customers", "load_warehouse").style("waiting"),
        DagEdge::new("validate", "audit_log").style("failed"),
        DagEdge::new("audit_log", "notify").style("skipped"),
    ];
    let styles = vec![
        EdgeStyle::new("ok", "Succeeded", Hue::Ok),
        EdgeStyle::new("active", "Running", Hue::Ice).animated(),
        EdgeStyle::new("waiting", "Waiting", Hue::Neutral).dashed(),
        EdgeStyle::new("failed", "Failed", Hue::Bad),
        EdgeStyle::new("skipped", "Skipped", Hue::Muted).dashed(),
    ];

    view! {
        <Dag
            nodes
            edges
            styles
            legend=true
            node_w=150.0
            node_h=52.0
            label="Workflow"
            on_select=Callback::new(move |id: String| selected.set(Some(id)))
            on_open=Callback::new(move |id: String| opened.set(Some(id)))
        />
        <p class="gallery__caption gallery__readout" data-testid="workflow-readout">
            "selected: " <b>{shown(selected)}</b> " · opened: " <b>{shown(opened)}</b>
        </p>
    }
}

/// Fixed layers with lane bands: the product gives the layer of each node,
/// and its lane (the parent). Only `blocks` edges are drawn.
#[component]
pub fn LanesDemo() -> impl IntoView {
    let selected = RwSignal::new(None::<String>);
    let opened = RwSignal::new(None::<String>);
    let more = RwSignal::new(None::<String>);

    let item = |id: &str, title: &str, layer: usize, status: &str| {
        let hue = match layer {
            0 => Hue::Violet,
            1 => Hue::Ice,
            _ => Hue::Teal,
        };
        DagNode::new(id, title)
            .sublabel(id)
            .detail(status)
            .layer(layer)
            .status(hue)
            .sort_key(id)
    };
    let nodes = vec![
        item("ACME-S-01", "Grow self-serve", 0, "active"),
        item("ACME-S-02", "Cut support load", 0, "active"),
        item("ACME-I-11", "Onboarding revamp", 1, "in progress")
            .lane("ACME-S-01")
            .current(true),
        item("ACME-I-12", "Usage billing", 1, "ready").lane("ACME-S-01"),
        item("ACME-I-21", "Help centre search", 1, "discovery").lane("ACME-S-02"),
        item("ACME-T-01", "Signup flow copy", 2, "done")
            .lane("ACME-I-11")
            .done(true)
            .mark("done", Hue::Ok),
        item("ACME-T-02", "Welcome email", 2, "doing").lane("ACME-I-11"),
        item("ACME-T-03", "Product tour", 2, "todo")
            .lane("ACME-I-11")
            .archived(true)
            .mark("put away", Hue::Gold),
        item("ACME-T-04", "Meter events", 2, "doing")
            .lane("ACME-I-12")
            .more(3),
        item("ACME-T-05", "Invoice preview", 2, "todo").lane("ACME-I-12"),
        item("ACME-T-06", "Index articles", 2, "doing").lane("ACME-I-21"),
        item("ACME-T-07", "Search box", 2, "todo").lane("ACME-I-21"),
    ];
    let lanes = [
        "ACME-S-01",
        "ACME-S-02",
        "ACME-I-11",
        "ACME-I-12",
        "ACME-I-21",
    ]
    .iter()
    .map(|id| DagLane::new(*id, *id).anchor(*id))
    .collect::<Vec<_>>();
    let edges = vec![
        DagEdge::new("ACME-T-01", "ACME-T-02").style("resolved"),
        DagEdge::new("ACME-T-04", "ACME-T-05").style("open"),
        DagEdge::new("ACME-T-05", "ACME-T-07").style("open"),
        DagEdge::new("ACME-T-06", "ACME-T-07").style("open"),
        DagEdge::new("ACME-I-12", "ACME-I-21").style("open"),
    ];
    let styles = vec![
        EdgeStyle::new("open", "Open blocker", Hue::Gold),
        EdgeStyle::new("resolved", "Resolved: one end is done", Hue::Muted).dashed(),
    ];
    let layers = vec!["Strategy".into(), "Initiative".into(), "Task".into()];

    view! {
        <Dag
            nodes
            edges
            lanes
            layers
            styles
            legend=true
            align=Align::Start
            node_w=200.0
            node_h=58.0
            label="Flight levels"
            on_select=Callback::new(move |id: String| selected.set(Some(id)))
            on_open=Callback::new(move |id: String| opened.set(Some(id)))
            on_more=Callback::new(move |id: String| more.set(Some(id)))
        />
        <p class="gallery__caption gallery__readout" data-testid="lanes-readout">
            "selected: " <b>{shown(selected)}</b> " · opened: " <b>{shown(opened)}</b>
            " · more: " <b>{shown(more)}</b>
        </p>
    }
}
