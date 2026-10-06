//! The repairs of AURORA-T-0008: the defects that the Cloacina migration to
//! 0.4.0 found, and 0.4.1 kept. One test per item, named by its number.
//!
//! As in `aurora_t_0007.rs`, a component renders to HTML on the server
//! renderer, and a repair that is pure CSS is checked in the stylesheet.

use aurora_leptos::components::*;
use aurora_leptos::COMPONENTS_CSS;
use leptos::prelude::*;

/// Renders a view to HTML, under a reactive owner.
macro_rules! html {
    ($($view:tt)*) => {{
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.with(|| view! { $($view)* }.to_html())
    }};
}

/// The body of the first CSS rule whose selector is exactly `selector`.
fn rule<'a>(selector: &str) -> &'a str {
    let css = COMPONENTS_CSS;
    let mut from = 0;
    while let Some(i) = css[from..].find(selector) {
        let at = from + i;
        let after = &css[at + selector.len()..];
        let starts_line = at == 0 || css[..at].ends_with('\n');
        if starts_line && after.trim_start().starts_with('{') {
            let body = &after[after.find('{').unwrap() + 1..];
            return &body[..body.find('}').expect("rule end")];
        }
        from = at + selector.len();
    }
    panic!("no rule {selector:?}")
}

fn css_pos(needle: &str) -> usize {
    COMPONENTS_CSS
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in components.css"))
}

// ---- 1. bad=true on the subtle and default variants -----------------------

#[test]
fn item_01_bad_colours_subtle_and_default_buttons() {
    for variant in ["subtle", "default"] {
        let base = format!(".cl-btn--bad.cl-btn--{variant}");
        assert!(rule(&base).contains("color: var(--bad-fg)"), "{variant}");
        assert!(
            rule(&format!("{base}:hover")).contains("background: var(--bad-bg)"),
            "{variant} hover"
        );
        // The same weight as `.cl-btn--{variant}:hover`, so it must come after.
        assert!(
            css_pos(&format!("{base}:hover")) > css_pos(&format!(".cl-btn--{variant}:hover")),
            "{variant} hover comes after the plain hover"
        );
    }
    let h = html! { <Button variant="subtle" bad=true>"Delete"</Button> };
    assert!(h.contains("cl-btn--subtle") && h.contains("cl-btn--bad"), "{h}");
}

// ---- 2. the placeholder of a password input -------------------------------

#[test]
fn item_02_password_placeholder_has_normal_spacing() {
    assert!(rule(r#".cl-input[type="password"]"#).contains("letter-spacing: 0.18em"));
    assert!(rule(r#".cl-input[type="password"]::placeholder"#).contains("letter-spacing: normal"));
}

// ---- 3. StatTile with a text value ----------------------------------------

#[test]
fn item_03_stat_tile_text_value_truncates() {
    assert!(rule(".cl-stat__value--text").contains("font-size: var(--fs-xl)"));
    let t = rule(".cl-stat__value--text .cl-stat__text");
    for decl in ["min-width: 0", "overflow: hidden", "text-overflow: ellipsis", "white-space: nowrap"] {
        assert!(t.contains(decl), "{decl}");
    }

    let h = html! { <StatTile label="Last deploy" value="just now" text=true /> };
    assert!(h.contains("cl-stat__value cl-stat__value--text"), "{h}");
    assert!(h.contains(r#"class="cl-stat__text""#) && h.contains(r#"title="just now""#), "{h}");

    // A figure keeps the big mono form.
    let h = html! { <StatTile label="Agents" value="3" /> };
    assert!(!h.contains("cl-stat__value--text") && h.contains(r#"class="cl-tnum""#), "{h}");
}
