//! The repairs of AURORA-T-0007: the defects that the Weir, Brokkr and Kairos
//! migrations to 0.4.0 found. One test (or more) per item, named by its
//! number.
//!
//! The components render to HTML on the server renderer (the `ssr` feature
//! of the leptos dev-dependency), so a test reads the markup. A repair that
//! is pure CSS is checked in the stylesheet: the rule is there, in the right
//! place. What only a browser shows (the cascade at run time, the timing of a
//! double click) is not tested here; the commit and the changelog say so.

use aurora_leptos::components::*;
use aurora_leptos::graph::*;
use aurora_leptos::widgets::*;
use aurora_leptos::{COMPONENTS_CSS, GRAPH_CSS, TOKENS_CSS};
use leptos::prelude::*;

/// Renders a view to HTML, under a reactive owner.
macro_rules! html {
    ($($view:tt)*) => {{
        // Once per process; a second call only says that it is set.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.with(|| view! { $($view)* }.to_html())
    }};
}

/// The first tag in `html` that contains `needle`, from `<` to `>`.
fn tag_with<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {html}"));
    let start = html[..at].rfind('<').expect("a tag start");
    let end = at + html[at..].find('>').expect("a tag end");
    &html[start..=end]
}

/// The body of the first CSS rule whose selector is exactly `selector`
/// (any space before the `{`).
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let mut from = 0;
    while let Some(i) = css[from..].find(selector) {
        let at = from + i;
        let after = &css[at + selector.len()..];
        let starts_line = at == 0 || css[..at].ends_with('\n') || css[..at].ends_with("  ");
        if starts_line && after.trim_start().starts_with('{') {
            let body = &after[after.find('{').unwrap() + 1..];
            return &body[..body.find('}').expect("rule end")];
        }
        from = at + selector.len();
    }
    panic!("no rule {selector:?}")
}

/// The text of each `@media (prefers-reduced-motion: reduce)` block.
fn reduced_motion_blocks(css: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = css;
    while let Some(at) = rest.find("@media (prefers-reduced-motion: reduce)") {
        let block = &rest[at..];
        // The block ends at the first "}" that closes it: a line with "}" alone.
        let end = block.find("\n}").map_or(block.len(), |e| e + 2);
        out.push(&block[..end]);
        rest = &block[end..];
    }
    out
}

// ---- 1. mono on TextInput and Textarea ------------------------------------

#[test]
fn item_01_mono_field_has_a_rule_after_cl_input() {
    let input = css_pos(".cl-input {");
    let mono = css_pos(".cl-input.cl-mono {");
    assert!(
        mono > input,
        "the mono rule must come after .cl-input, or .cl-input wins"
    );
    assert!(rule(COMPONENTS_CSS, ".cl-input.cl-mono").contains("font-family: var(--font-mono)"));

    let v = RwSignal::new(String::new());
    let h = html! { <TextInput value=v mono=true /> <Textarea value=v mono=true /> };
    assert!(tag_with(&h, "<input").contains("cl-input cl-mono"));
    assert!(tag_with(&h, "<textarea").contains("cl-input cl-mono"));
}

fn css_pos(needle: &str) -> usize {
    COMPONENTS_CSS
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in components.css"))
}

// ---- 2. Select has an aria_label ------------------------------------------

#[test]
fn item_02_select_takes_an_accessible_name() {
    let v = RwSignal::new(String::from("a"));
    let h = html! { <Select options=vec!["a".into()] value=v aria_label="Board" /> };
    assert!(tag_with(&h, "<select").contains(r#"aria-label="Board""#));
    // With no aria_label, no empty attribute.
    let h = html! { <Select options=vec!["a".into()] value=v label="Board" /> };
    assert!(!tag_with(&h, "<select").contains("aria-label"));
}

// ---- 3. SimpleGrid collapses on a small screen ----------------------------

#[test]
fn item_03_simple_grid_collapses() {
    let h = html! { <SimpleGrid cols=4><div>"a"</div></SimpleGrid> };
    let tag = tag_with(&h, "cl-simple-grid");
    assert!(tag.contains("--cl-cols:4"), "{tag}");
    assert!(
        !tag.contains("grid-template-columns"),
        "an inline column count beats every media query: {tag}"
    );
    let at_480 = &COMPONENTS_CSS[css_pos("@media (max-width: 480px) {\n  .cl-simple-grid")..];
    assert!(at_480.contains("grid-template-columns: minmax(0, 1fr)"));
    let h = html! { <SimpleGrid cols=3 fixed=true><div>"a"</div></SimpleGrid> };
    assert!(tag_with(&h, "cl-simple-grid").contains("cl-simple-grid--fixed"));
}

// ---- 4. The title of Modal and ConfirmDialog is reactive ------------------

#[test]
fn item_04_dialog_titles_take_a_signal() {
    let open = RwSignal::new(true);
    let name = RwSignal::new(String::from("fx-demo"));
    let h = html! {
        <ConfirmDialog open title=move || format!("Delete {}?", name.get())
            on_confirm=Callback::new(|_| {}) />
    };
    assert!(h.contains("Delete fx-demo?"), "{h}");
    assert!(tag_with(&h, r#"role="dialog""#).contains(r#"aria-label="Delete fx-demo?""#));

    let title = Signal::derive(move || format!("Edit {}", name.get()));
    let h = html! { <Modal open title>"body"</Modal> };
    assert!(h.contains("Edit fx-demo"));
    // A plain string still works (the 0.4.0 call shape).
    let h = html! { <Drawer open title="Details">"body"</Drawer> };
    assert!(h.contains("Details"));
}

// ---- 5. .cl-pulse stops when the OS asks for less motion ------------------

#[test]
fn item_05_pulse_has_a_reduced_motion_guard() {
    let guarded = reduced_motion_blocks(TOKENS_CSS)
        .iter()
        .any(|b| b.contains(".cl-pulse { animation: none; }"));
    assert!(
        guarded,
        "tokens.css must stop .cl-pulse under reduced motion"
    );
}

// ---- 6. FeedRow with no time has no time slot -----------------------------

#[test]
fn item_06_feed_row_without_time_has_no_slot() {
    let h = html! { <FeedRow subject="fx">"text"</FeedRow> };
    assert!(!h.contains("cl-feed__time"), "{h}");
    assert!(h.contains("cl-feed__row--no-time"));

    let h = html! { <FeedRow time="09:12" subject="fx">"text"</FeedRow> };
    assert!(h.contains("cl-feed__time"));
    assert!(!h.contains("cl-feed__row--no-time"));

    let h = html! { <FeedRow keep_time_slot=true subject="fx">"text"</FeedRow> };
    assert!(h.contains("cl-feed__time"), "the opt-in keeps the column");
}

// ---- 7. A long sub line of StatTile wraps ---------------------------------

#[test]
fn item_07_stat_sub_line_wraps() {
    let sub = rule(COMPONENTS_CSS, ".cl-stat__sub, .cl-stat__delta");
    assert!(sub.contains("min-width: 0") && sub.contains("overflow-wrap: anywhere"));
    assert!(rule(COMPONENTS_CSS, ".cl-stat").contains("min-width: 0"));
    assert!(rule(COMPONENTS_CSS, ".cl-stat__foot").contains("min-width: 0"));
    let h = html! { <StatTile label="Agents" value="3" sub="a-very-long-line-with-no-space" /> };
    assert!(h.contains(r#"<span class="cl-stat__sub">"#));
}

// ---- 8. Select with an unknown value shows the first option ---------------

#[test]
fn item_08_select_shows_the_first_option_for_an_unknown_value() {
    let opts = vec!["a".to_string(), "b".to_string()];
    assert_eq!(select_shown_value("zzz", &opts, false), "a");
    assert_eq!(select_shown_value("b", &opts, false), "b");
    assert_eq!(select_shown_value("zzz", &opts, true), "zzz", "placeholder");
    assert_eq!(select_shown_value("x", &[], false), "x");

    let v = RwSignal::new(String::from("zzz"));
    let h = html! { <Select options=opts.clone() value=v /> };
    assert!(
        tag_with(&h, r#"<option value="a""#).contains("selected"),
        "{h}"
    );
    assert_eq!(v.get_untracked(), "zzz", "the value is not changed");
}

// ---- 9. An indeterminate progress bar --------------------------------------

#[test]
fn item_09_meter_indeterminate() {
    let h = html! { <Meter indeterminate=true label="Collecting diagnostics" /> };
    let tag = tag_with(&h, "cl-meter--indeterminate");
    assert!(tag.contains(r#"role="progressbar""#), "{tag}");
    assert!(tag.contains(r#"aria-label="Collecting diagnostics""#));
    assert!(
        !tag.contains("aria-valuenow"),
        "unknown length has no value"
    );
    let h = html! { <Meter indeterminate=true /> };
    assert!(h.contains(r#"aria-label="Loading""#));
    // The 0.4.0 call shape still works.
    let h = html! { <Meter value=40.0 label="Disk" /> };
    assert!(h.contains(r#"aria-valuenow="40""#));
    let still = reduced_motion_blocks(COMPONENTS_CSS)
        .iter()
        .any(|b| b.contains(".cl-meter--indeterminate .cl-meter__fill { animation: none;"));
    assert!(still, "the sweep stops under reduced motion");
}

// ---- 10. Chip is a toggle button ------------------------------------------

#[test]
fn item_10_chip_type_and_pressed() {
    let h = html! { <Chip label="parent" active=true /> };
    let tag = tag_with(&h, "cl-chip");
    assert!(tag.contains(r#"type="button""#), "{tag}");
    assert!(tag.contains(r#"aria-pressed="true""#));
    let h = html! { <Chip label="child" active=false /> };
    assert!(tag_with(&h, "cl-chip").contains(r#"aria-pressed="false""#));
}

// ---- 11. Group aligns at the bottom ---------------------------------------

#[test]
fn item_11_group_align_end() {
    let h = html! { <Group align="end"><span>"a"</span></Group> };
    assert!(h.contains("cl-group cl-group--bottom"), "{h}");
    assert!(rule(COMPONENTS_CSS, ".cl-group--bottom").contains("align-items: flex-end"));
    let h = html! { <Group top=true><span>"a"</span></Group> };
    assert!(h.contains("cl-group--top"), "top still works");
}

// ---- 12. error is reactive; inputs take an id -------------------------------

#[test]
fn item_12_error_is_reactive_and_id_is_settable() {
    let v = RwSignal::new(String::new());
    let err = RwSignal::new(String::from("Required"));
    let h = html! { <TextInput value=v error=err id="board-name" /> };
    let input = tag_with(&h, "<input");
    assert!(input.contains(r#"id="board-name""#), "{input}");
    assert!(input.contains("cl-input--error"));
    assert!(input.contains(r#"aria-invalid="true""#));
    assert!(input.contains(r#"aria-describedby="board-name-error""#));
    let msg = tag_with(&h, "cl-field__error");
    assert!(msg.contains(r#"id="board-name-error""#), "{msg}");
    assert!(h.contains("Required</span>"));

    err.set(String::new());
    let h = html! { <TextInput value=v error=err /> };
    assert!(!h.contains("cl-field__error"));
    assert!(!tag_with(&h, "<input").contains("aria-invalid"));

    let h = html! { <Select options=vec!["a".into()] value=v id="pick" error=move || "Pick one".to_string() /> };
    assert!(tag_with(&h, "<select").contains(r#"id="pick""#));
    assert!(h.contains("Pick one"));
    // A plain string still works (the 0.4.0 call shape).
    let h = html! { <TextInput value=v error="Too long" /> };
    assert!(h.contains("Too long"));
}

// ---- 13. A sticky header sets scroll-padding-top --------------------------

#[test]
fn item_13_appshell_sets_scroll_padding() {
    let sel = "html:has(.cl-appshell:not(.cl-appshell--contained):not(.cl-appshell--no-header))";
    assert!(rule(COMPONENTS_CSS, sel).contains("scroll-padding-top"));
}

// ---- 14. Pagination range text: hide or replace ---------------------------

#[test]
fn item_14_pagination_range_text() {
    let offset = RwSignal::new(40usize);
    let limit = RwSignal::new(20usize);
    let h = html! { <Pagination offset limit total=212usize /> };
    assert!(h.contains("41–60 of 212"));
    let h = html! { <Pagination offset limit total=212usize show_range=false /> };
    assert!(!h.contains("cl-pager__range"), "{h}");
    let h = html! {
        <Pagination offset limit total=212usize
            range_label=Callback::new(|r: PageRange| format!("Rows {} to {}", r.first, r.last)) />
    };
    assert!(h.contains("Rows 41 to 60"));
}

// ---- 15. Dag edges have a style name; marker ids are stable ---------------

fn small_dag(id: &'static str) -> String {
    let nodes = vec![DagNode::new("a", "A"), DagNode::new("b", "B")];
    let edges = vec![DagEdge::new("a", "b").style("blocks")];
    let styles = vec![EdgeStyle::new("blocks", "Blocks", Hue::Gold)];
    html! { <Dag nodes edges styles id=id /> }
}

#[test]
fn item_15_edges_have_a_style_and_markers_a_stable_id() {
    let h = small_dag("plan");
    let edge = tag_with(&h, r#"data-from="a""#);
    assert!(edge.contains(r#"data-style="blocks""#), "{edge}");
    assert!(edge.contains(r#"data-from="a""#) && edge.contains(r#"data-to="b""#));
    let marker = dag_marker_id("plan", "blocks");
    assert_eq!(marker, "plan-arrow-blocks");
    assert!(edge.contains(&format!("url(#{marker})")));
    assert!(h.contains(&format!(r#"<marker id="{marker}""#)));
    assert_eq!(small_dag("plan"), h, "the same graph gives the same ids");
    assert_eq!(dag_marker_id("g", "Blocked by"), "g-arrow-blocked-by");
}

// ---- 16. A double click opens without a select ----------------------------

#[test]
fn item_16_double_click_intent() {
    // Without defer_select: the first click selects; the second click of a
    // double click no longer selects again.
    assert_eq!(click_intent(1, false), ClickIntent::SelectNow);
    assert_eq!(click_intent(2, false), ClickIntent::Ignore);
    // With it: the first click waits; the dblclick cancels it and opens.
    assert_eq!(click_intent(1, true), ClickIntent::SelectLater);
    assert_eq!(click_intent(2, true), ClickIntent::Ignore);
    // A keyboard or synthetic click (detail 0) acts at once.
    assert_eq!(click_intent(0, false), ClickIntent::SelectNow);
    const { assert!(DOUBLE_CLICK_MS > 0) };
    let nodes = vec![DagNode::new("a", "A")];
    let _ = html! { <Dag nodes edges=vec![] defer_select=true /> };
}

// ---- 17. The "+N" badge belongs to its node -------------------------------

#[test]
fn item_17_more_badge_is_in_the_node_item() {
    let mut node = DagNode::new("a", "A");
    node.more = 3;
    let h = html! { <Dag nodes=vec![node] edges=vec![] /> };
    assert!(tag_with(&h, r#"class="cl-dag__item""#).contains(r#"data-node="a""#));
    assert!(tag_with(&h, r#"class="cl-dag__more""#).contains(r#"data-node="a""#));
    let item = h.find(r#"class="cl-dag__item""#).expect("item");
    let node_at = h.find(r#"data-id="a""#).expect("node");
    let more_at = h.find(r#"class="cl-dag__more""#).expect("badge");
    assert!(item < node_at && node_at < more_at, "{h}");
    // The node's group closes after the badge: nothing between them but
    // the node's own content.
    assert!(!h[node_at..more_at].contains("cl-dag__item"));
    assert!(GRAPH_CSS.contains(
        ".cl-dag--hovering .cl-dag__node:not(.cl-dag__node--hot):not(.cl-dag__node--near) + .cl-dag__more { opacity: 0.45; }"
    ));
}

// ---- 18. ConfirmDialog: a notice before the impacts list ------------------

#[test]
fn item_18_notice_comes_before_the_impacts() {
    let open = RwSignal::new(true);
    let impacts = Signal::derive(|| vec!["FX-T-1".to_string()]);
    let h = html! {
        <ConfirmDialog open title="Archive?" impacts on_confirm=Callback::new(|_| {})
            notice=std::sync::Arc::new(|| view! { <p>"WARNING"</p> }.into_any())>
            <p>"AFTER"</p>
        </ConfirmDialog>
    };
    let notice = h.find("WARNING").expect("notice");
    let list = h.find("cl-confirm__impacts").expect("impacts");
    let after = h.find("AFTER").expect("children");
    assert!(notice < list && list < after, "{h}");
}

// ---- 19. The README trunk path works without a global install ------------

#[test]
fn item_19_readme_gives_a_trunk_path_with_no_install() {
    let readme = include_str!("../../README.md");
    // The helper crate: leptos-free, and it calls write_css.
    assert!(
        readme.contains(r#"colliery-io-aurora = { version = "0.4", default-features = false }"#)
    );
    assert!(readme.contains("aurora_leptos::write_css(std::path::Path::new(&dir))"));
    assert!(readme.contains(r#""run", "-q", "-p", "aurora-css","#));
    // The install line pins the version of this crate.
    let pin = format!("--locked --version {}", env!("CARGO_PKG_VERSION"));
    assert!(readme.contains(&pin), "README must say {pin}");
}

// ---- 20. Empty with a next step -------------------------------------------

#[test]
fn item_20_empty_has_a_next_step() {
    let h = html! {
        <Empty message="No agents yet." hint="Install an agent to see it here."
            href="https://example.test/agents" />
    };
    assert!(h.contains("No agents yet."));
    let next = &h[h.find("cl-empty__next").expect("next step")..];
    assert!(next.contains("Install an agent to see it here."));
    assert!(next.contains(r#"href="https://example.test/agents""#));
    assert!(next.contains("Read how"));
    let h = html! { <Empty message="Nothing here."><button>"Create"</button></Empty> };
    assert!(
        h.contains(r#"<div class="cl-empty__actions"><button>Create</button></div>"#),
        "{h}"
    );
    // The 0.4.0 call shape: only a message, no next-step markup.
    let h = html! { <Empty message="None." /> };
    assert!(!h.contains("cl-empty__next") && !h.contains("cl-empty__actions"));
}
