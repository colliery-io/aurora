//! AURORA-T-0012: the stylesheet of the Hlin pack has the rules of each
//! component that `AuroraPack` draws.
//!
//! Each view of the pack renders to HTML on the server renderer. Each `cl-`
//! class in it that `AURORA_CSS` styles must be styled in `HLIN_CSS` too.
#![cfg(feature = "hlin")]

use aurora_leptos::{AuroraPack, AURORA_CSS, HLIN_CSS};
use hlin_view::envelope::Envelope;
use hlin_view::{Cause, Context, DesignPack, Emit, Kind, Notice, Treatment};
use leptos::prelude::*;

fn context(notice: Option<Cause>) -> Context {
    Context {
        treatment: Treatment::Normal,
        notice: notice.map(|cause| Notice {
            cause,
            offer_successor: false,
        }),
        age_seconds: None,
        controls: Vec::new(),
        emit: Emit::nowhere(),
    }
}

fn envelope(json: &str) -> Envelope {
    serde_json::from_str(json).expect("envelope")
}

const SCALAR: &str =
    r#"{"envelope":"scalar.v1","value":42,"unit":"ms","label":"p95","previous":40}"#;
const SERIES: &str = r#"{"envelope":"series.v1","unit":"rps","series":[
    {"name":"a","points":[[1000,1],[2000,3],[3000,null],[4000,2]]},
    {"name":"b","points":[[1000,2],[2000,2],[3000,4],[4000,1]]}]}"#;
const RECORDS: &str = r#"{"envelope":"records.v1","columns":[
    {"key":"id","label":"Id","type":"string"},
    {"key":"label","label":"Label","type":"string"},
    {"key":"depends_on","label":"From","type":"string"},
    {"key":"state","label":"State","type":"string"},
    {"key":"n","label":"N","type":"number"}],
    "rows":[{"id":"db","label":"db","state":"ok","n":1},
            {"id":"api","depends_on":"db","state":"degraded","n":2},
            {"id":"web","depends_on":"api","state":"down","n":3}]}"#;
const STATUS: &str = r#"{"envelope":"status.v1","status":"degraded","label":"cluster","detail":"one node down",
    "items":[{"name":"a","status":"ok"},{"name":"b","status":"down","detail":"gone"}]}"#;
const OPTIONS: &str = r#"{"envelope":"options.v1","options":[{"value":"a","label":"A"}]}"#;

/// The HTML of every view the pack draws.
fn every_view() -> String {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let pack = AuroraPack;
        let mut views: Vec<AnyView> = Vec::new();
        let (scalar, series, records, status, options) = (
            envelope(SCALAR),
            envelope(SERIES),
            envelope(RECORDS),
            envelope(STATUS),
            envelope(OPTIONS),
        );
        let (
            Envelope::Scalar(sc),
            Envelope::Series(se),
            Envelope::Records(re),
            Envelope::Status(st),
            Envelope::Options(op),
        ) = (&scalar, &series, &records, &status, &options)
        else {
            panic!("envelope shapes")
        };
        views.push(pack.stat(sc, context(None)));
        views.push(pack.timeseries(se, context(None)));
        views.push(pack.sparkline(se, context(None)));
        views.push(pack.table(re, context(None)));
        views.push(pack.series_as_table(se, context(None)));
        views.push(pack.status(st, context(None)));
        views.push(pack.options(op, context(None)));
        for document in [&scalar, &series, &records, &status, &options] {
            views.push(pack.raw(document, context(None)));
        }
        for component in pack.offers() {
            for document in [&scalar, &series, &records] {
                if let Some(view) = pack.custom(component, document, context(None)) {
                    views.push(view);
                }
            }
        }
        views.push(pack.skeleton(Kind::Table, context(None)));
        views.push(pack.placeholder(Kind::Table, context(None)));
        for cause in [Cause::Forbidden, Cause::Unreachable, Cause::Unknown] {
            views.push(pack.placeholder(Kind::Table, context(Some(cause))));
        }
        views.into_iter().map(|view| view.to_html()).collect()
    })
}

/// The `cl-` classes in the `class` attributes of some HTML.
fn classes(html: &str) -> std::collections::BTreeSet<String> {
    let mut found = std::collections::BTreeSet::new();
    for part in html.split("class=\"").skip(1) {
        let value = &part[..part.find('"').unwrap_or(0)];
        for class in value.split_whitespace() {
            if class.starts_with("cl-") {
                found.insert(class.to_string());
            }
        }
    }
    found
}

/// Whether a stylesheet has a selector with this class.
fn styles(css: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    css.match_indices(&needle).any(|(at, _)| {
        let next = css[at + needle.len()..].chars().next().unwrap_or(' ');
        !(next.is_ascii_alphanumeric() || next == '-' || next == '_')
    })
}

#[test]
fn hlin_css_styles_each_component_the_pack_draws() {
    let html = every_view();
    let used = classes(&html);
    assert!(used.contains("cl-dag"), "the graph is drawn: {used:?}");
    let missing: Vec<&String> = used
        .iter()
        .filter(|class| styles(AURORA_CSS, class) && !styles(HLIN_CSS, class))
        .collect();
    assert!(
        missing.is_empty(),
        "styled in AURORA_CSS, not in HLIN_CSS: {missing:?}"
    );
}

#[test]
fn hlin_css_has_the_graph_stylesheet() {
    assert!(HLIN_CSS.contains(aurora_leptos::GRAPH_CSS));
}
