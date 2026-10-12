//! The repairs of AURORA-T-0013: the defects that the Skadi migration to
//! 0.5.0 found. One test per item, named by its number.
//!
//! As in `aurora_t_0007.rs`, a component renders to HTML on the server
//! renderer, and a repair that is pure CSS is checked in the stylesheet. What
//! only a browser does (the drawer closing on a resize, a dialog moving to
//! `<body>`, focus) is checked in the gallery.

use aurora_leptos::components::*;
use aurora_leptos::{COMPONENTS_CSS, TOKENS_CSS};
use leptos::prelude::*;

/// Renders a view to HTML, under a reactive owner.
macro_rules! html {
    ($($view:tt)*) => {{
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.with(|| view! { $($view)* }.to_html())
    }};
}

/// The body of the first CSS rule whose selector list contains `selector` as
/// a whole line, or as the whole selector.
fn rule<'a>(selector: &str) -> &'a str {
    let css = COMPONENTS_CSS;
    let mut from = 0;
    while let Some(i) = css[from..].find(selector) {
        let at = from + i;
        let after = &css[at + selector.len()..];
        let starts_line = at == 0 || css[..at].ends_with('\n') || css[..at].ends_with("  ");
        let rest = after.trim_start();
        if starts_line && (rest.starts_with('{') || rest.starts_with(',')) {
            let body = &after[after.find('{').unwrap() + 1..];
            return &body[..body.find('}').expect("rule end")];
        }
        from = at + selector.len();
    }
    panic!("no rule {selector:?}")
}

/// The opening tag of the first element whose tag text contains `needle`.
fn tag<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {html}"));
    let start = html[..=at].rfind('<').unwrap();
    let end = at + html[at..].find('>').unwrap();
    &html[start..=end]
}

/// A component that reads the drawer of the shell above it.
#[component]
fn DrawerProbe(expect: RwSignal<bool>) -> impl IntoView {
    let nav = use_app_shell().expect("AppShell provides AppShellNav");
    assert!(
        nav.open_signal() == expect,
        "the shell uses the signal it was given"
    );
    nav.close();
    view! { <span>"probe"</span> }
}

// ---- 1. AppShell gives its drawer state to the product --------------------

#[test]
fn item_01_appshell_exposes_the_drawer_and_locks_the_page() {
    let open = RwSignal::new(true);
    let h = html! {
        <AppShell nav_open=open navbar=Box::new(|| view! { <a href="/x">"x"</a> }.into_any())>
            <DrawerProbe expect=open />
        </AppShell>
    };
    assert!(h.contains("probe"));
    // DrawerProbe closed it before the class was read.
    assert!(!h.contains("cl-appshell--nav-open"), "{h}");

    let open = RwSignal::new(true);
    let h = html! {
        <AppShell nav_open=open navbar=Box::new(|| view! { <a href="/x">"x"</a> }.into_any())>
            "page"
        </AppShell>
    };
    assert!(h.contains("cl-appshell--nav-open"), "{h}");
    // The page under an open drawer does not scroll.
    assert!(COMPONENTS_CSS.contains(
        "html:has(.cl-appshell--nav-open:not(.cl-appshell--contained):not(.cl-appshell--bp)) { overflow: hidden; }"
    ));
}

// ---- 2. AppShell's <main> takes an id and a class --------------------------

#[test]
fn item_02_main_takes_an_id_a_class_and_focus() {
    let h = html! { <AppShell main_id="content" main_class="page">"x"</AppShell> };
    let main = tag(&h, "<main");
    assert!(main.contains(r#"class="cl-appshell__main page""#), "{main}");
    assert!(main.contains(r#"id="content""#), "{main}");
    assert!(main.contains(r#"tabindex="-1""#), "{main}");
    // With no id, no tabindex either.
    let h = html! { <AppShell>"x"</AppShell> };
    let main = tag(&h, "<main");
    assert!(
        !main.contains("id=") && !main.contains("tabindex"),
        "{main}"
    );
}

// ---- 3. tokens.css does not style #main ------------------------------------

#[test]
fn item_03_tokens_do_not_style_main() {
    let rules = TOKENS_CSS
        .lines()
        .filter(|l| !l.trim_start().starts_with("/*"))
        .any(|l| l.contains("#main"));
    assert!(!rules, "a rule in tokens.css names #main");
}

// ---- 4. A skip link before the header --------------------------------------

#[test]
fn item_04_skip_link_is_the_first_element() {
    let h = html! {
        <AppShell skip_link="Skip to content" header=Box::new(|| view! { "bar" }.into_any())>
            "x"
        </AppShell>
    };
    let shell_end = h.find('>').unwrap();
    let first = &h[shell_end + 1..];
    let link = tag(first, "<a");
    assert!(first.starts_with(link), "{h}");
    assert!(
        link.contains(r#"class="cl-skip-link""#) && link.contains(r##"href="#main""##),
        "{link}"
    );
    assert!(
        first[link.len()..].starts_with("Skip to content</a>"),
        "{h}"
    );
    assert!(tag(&h, "<main").contains(r#"id="main""#));
    assert!(rule(".cl-skip-link:focus").contains("transform: none"));
}

// ---- 5. The breakpoint is a prop -------------------------------------------

#[test]
fn item_05_breakpoint_is_a_prop() {
    assert_eq!(APPSHELL_BREAKPOINT, 768);
    let h = html! { <AppShell breakpoint=1024>"x"</AppShell> };
    assert!(tag(&h, "cl-appshell").contains("cl-appshell--bp"), "{h}");
    // The narrow rules exist for a shell with its own breakpoint, and the
    // media query leaves that shell alone.
    assert!(
        rule(".cl-appshell--bp.cl-appshell--narrow .cl-appshell__navbar")
            .contains("position: fixed")
    );
    assert!(
        rule(".cl-appshell:not(.cl-appshell--bp) .cl-appshell__navbar").contains("position: fixed")
    );
    // The two lists are the same, line for line.
    let media = COMPONENTS_CSS
        .lines()
        .filter(|l| l.starts_with("  .cl-appshell:not(.cl-appshell--bp)"))
        .map(|l| l.trim().replace(".cl-appshell:not(.cl-appshell--bp)", "X"))
        .collect::<Vec<_>>();
    let bp = COMPONENTS_CSS
        .lines()
        .filter(|l| l.starts_with(".cl-appshell--bp.cl-appshell--narrow"))
        .map(|l| {
            l.trim()
                .replace(".cl-appshell--bp.cl-appshell--narrow", "X")
        })
        .collect::<Vec<_>>();
    assert!(!media.is_empty());
    assert_eq!(media, bp);
}

// ---- 6. The README names the current version -------------------------------

#[test]
fn item_06_readmes_name_the_current_version() {
    let minor = env!("CARGO_PKG_VERSION").rsplit_once('.').unwrap().0;
    let line = format!(r#"colliery-io-aurora = "{minor}""#);
    for (name, text) in [
        ("README.md", include_str!("../../README.md")),
        ("aurora-leptos/README.md", include_str!("../README.md")),
    ] {
        assert!(text.contains(&line), "{name} must say {line}");
        for old in ["0.1", "0.2", "0.3", "0.4"] {
            assert!(
                !text.contains(&format!(r#"colliery-io-aurora = "{old}""#))
                    && !text.contains(&format!(r#"colliery-io-aurora = {{ version = "{old}""#)),
                "{name} names {old}"
            );
        }
    }
}

// ---- 7. The marked segment is clear ----------------------------------------

#[test]
fn item_07_active_segment_is_clear() {
    let r = rule(".cl-segmented__item--active");
    assert!(r.contains("background: var(--ice-bg)"), "{r}");
    assert!(r.contains("color: var(--ice-fg)"), "{r}");
    assert!(r.contains("box-shadow: inset 0 0 0 1px"), "{r}");
    // tests/contrast.rs checks --ice-fg on --ice-bg over --inset in each theme.
}

// ---- 8. Segmented items have a font and a focus ring -----------------------

#[test]
fn item_08_segmented_items_have_font_and_ring() {
    assert!(rule(".cl-segmented__item").contains("font: inherit"));
    assert!(rule(".cl-segmented__item:focus-visible").contains("outline: 2px solid var(--ice)"));
}

// ---- 9. Fields: help line, required, aria_label, autofocus, limits ---------

#[test]
fn item_09_fields_link_a_help_line_and_take_limits() {
    let v = RwSignal::new(String::new());
    let h = html! {
        <TextInput label="Name" value=v id="n" description="Shown on the card." required=true
            autofocus=true maxlength=40 />
    };
    let input = tag(&h, "<input");
    assert!(input.contains(r#"aria-describedby="n-help""#), "{input}");
    assert!(
        input.contains("required") && input.contains("autofocus"),
        "{input}"
    );
    assert!(input.contains(r#"maxlength="40""#), "{input}");
    let help = tag(&h, "cl-field__help");
    assert!(help.contains(r#"id="n-help""#), "{help}");
    assert!(
        h.contains(&format!("{help}Shown on the card.</span>")),
        "{h}"
    );
    let mark = tag(&h, "cl-field__required");
    assert!(mark.contains(r#"aria-hidden="true""#), "{mark}");
    assert!(h.contains(&format!("{mark} *</span>")), "{h}");

    // The help line, the ids a product gives and the error, in that order.
    let h = html! {
        <TextInput value=v id="m" description="Help" described_by="elsewhere" error="Wrong"
            input_type="number" min="1" max="9" step="2" />
    };
    let input = tag(&h, "<input");
    assert!(
        input.contains(r#"aria-describedby="m-help elsewhere m-error""#),
        "{input}"
    );
    assert!(
        input.contains(r#"min="1""#)
            && input.contains(r#"max="9""#)
            && input.contains(r#"step="2""#)
    );

    let h = html! { <Select value=v id="s" options=vec!["a".to_string()] description="Pick one" required=true /> };
    assert!(
        tag(&h, "<select").contains(r#"aria-describedby="s-help""#),
        "{h}"
    );
    assert!(tag(&h, "<select").contains("required"), "{h}");

    let h = html! { <Textarea value=v id="t" description="Markdown" maxlength=500 /> };
    let area = tag(&h, "<textarea");
    assert!(
        area.contains(r#"aria-describedby="t-help""#) && area.contains(r#"maxlength="500""#),
        "{area}"
    );

    let h = html! { <PasswordInput value=v aria_label="Password" /> };
    assert!(
        tag(&h, "<input").contains(r#"aria-label="Password""#),
        "{h}"
    );
}

// ---- 10. Chip's count can change -------------------------------------------

#[test]
fn item_10_chip_count_is_a_signal() {
    let n = RwSignal::new(3);
    let h = html! { <Chip label="Failed" count=n active=false /> };
    assert!(
        h.contains(r#"<span class="cl-chip__count">3</span>"#),
        "{h}"
    );
    let h = html! { <Chip label="Failed" count=move || n.get() * 2 active=false /> };
    assert!(h.contains(">6</span>"), "{h}");
    // A plain number still works, and -1 (the default) shows no count.
    let h = html! { <Chip label="All" count=0 active=true /> };
    assert!(h.contains(">0</span>"), "{h}");
    let h = html! { <Chip label="All" active=true /> };
    assert!(!h.contains("cl-chip__count"), "{h}");
}

// ---- 11. Select's options can change ---------------------------------------

#[test]
fn item_11_select_options_are_a_signal() {
    let v = RwSignal::new("b".to_string());
    let opts = RwSignal::new(vec![("a".to_string(), "A".to_string())]);
    opts.update(|o| o.push(("b".to_string(), "B".to_string())));
    let h = html! { <Select value=v option_pairs=opts /> };
    assert!(h.contains(">B</option>"), "{h}");
    let h = html! { <Select value=v options=move || vec!["x".to_string(), "b".to_string()] /> };
    assert!(h.contains(">x</option>"), "{h}");
}

// ---- 12. ActionIcon: a name, type="button", disabled -----------------------

#[test]
fn item_12_action_icon_is_a_named_button() {
    let off = RwSignal::new(true);
    let h = html! { <ActionIcon aria_label="Delete" title="Delete" disabled=off>"×"</ActionIcon> };
    let b = tag(&h, "<button");
    assert!(b.contains(r#"type="button""#), "{b}");
    assert!(b.contains(r#"aria-label="Delete""#), "{b}");
    assert!(b.contains("disabled"), "{b}");
    assert!(rule(".cl-action-icon:disabled").contains("cursor: not-allowed"));
}

// ---- 13. Button's title and aria_label can change --------------------------

#[test]
fn item_13_button_labels_are_signals() {
    let name = RwSignal::new("run 7".to_string());
    let h = html! {
        <Button title=move || format!("Open {}", name.get()) aria_label=move || format!("Open {}", name.get())>
            "Open"
        </Button>
    };
    let b = tag(&h, "<button");
    assert!(
        b.contains(r#"title="Open run 7""#) && b.contains(r#"aria-label="Open run 7""#),
        "{b}"
    );
    // A fixed string still works, and an empty one is left out.
    let h = html! { <Button title="Save">"Save"</Button> };
    let b = tag(&h, "<button");
    assert!(
        b.contains(r#"title="Save""#) && !b.contains("aria-label"),
        "{b}"
    );
}

// ---- 14. Empty's link is underlined ----------------------------------------

#[test]
fn item_14_empty_link_is_not_colour_alone() {
    assert!(rule(".cl-empty__next .cl-anchor").contains("text-decoration: underline"));
    let h = html! { <Empty message="No runs." hint="Start one." href="/new" link="New run" /> };
    assert!(
        h.contains(r#"<p class="cl-empty__next">"#) && h.contains("cl-anchor"),
        "{h}"
    );
}

// ---- 15. Toasts are read from persistent live regions ----------------------

#[test]
fn item_15_toast_stack_has_persistent_live_regions() {
    let t = Toaster::new();
    t.error("Disk full");
    let h = html! { <ToastStack toaster=t /> };
    for (live, role) in [("polite", "status"), ("assertive", "alert")] {
        let region = tag(&h, &format!(r#"aria-live="{live}""#));
        assert!(region.contains(&format!(r#"role="{role}""#)), "{region}");
        assert!(region.contains(r#"aria-atomic="true""#), "{region}");
        assert!(region.contains(r#"class="cl-sr-only""#), "{region}");
        // Before the stack, so it is on the page from the start.
        assert!(
            h.find(region).unwrap() < h.find("cl-toasts").unwrap(),
            "{h}"
        );
    }
    // The toast on screen is not a live region of its own.
    let toast = tag(&h, "cl-toast cl-toast--error");
    assert!(!toast.contains("role="), "{toast}");
}

// ---- 16. Dialogs leave the page's layout -----------------------------------

#[test]
fn item_16_dialogs_have_in_place() {
    // Moving the layer to <body> needs a browser (checked in the gallery). On
    // the server the dialog renders where it is, and `in_place` keeps it there
    // in the browser too.
    let open = RwSignal::new(true);
    let h = html! { <Modal open title="T" in_place=true>"body"</Modal> };
    assert!(h.contains("cl-modal-overlay"), "{h}");
    let h = html! { <Drawer open title="T">"body"</Drawer> };
    assert!(h.contains("cl-drawer-overlay"), "{h}");
}

// ---- 17. ConfirmDialog: focus and alertdialog ------------------------------

#[test]
fn item_17_confirm_dialog_focus_and_role() {
    let open = RwSignal::new(true);
    let h = html! {
        <ConfirmDialog open title="Stop the server?" message="Streams end now." alert=true
            initial_focus="confirm" on_confirm=Callback::new(|_| {}) />
    };
    let dialog = tag(&h, r#"aria-modal="true""#);
    assert!(dialog.contains(r#"role="alertdialog""#), "{dialog}");
    let msg_id = tag(&h, "cl-confirm__message")
        .split(r#"id=""#)
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("message id")
        .to_string();
    assert!(
        dialog.contains(&format!(r#"aria-describedby="{msg_id}""#)),
        "{dialog}"
    );
    assert!(tag(&h, "cl-btn--filled").contains("data-autofocus"), "{h}");
    assert!(
        !tag(&h, "cl-btn--default").contains("data-autofocus"),
        "{h}"
    );

    // The default: a dialog, and Cancel has focus.
    let h = html! { <ConfirmDialog open title="Delete?" on_confirm=Callback::new(|_| {}) /> };
    assert!(
        tag(&h, r#"aria-modal="true""#).contains(r#"role="dialog""#),
        "{h}"
    );
    assert!(tag(&h, "cl-btn--default").contains("data-autofocus"), "{h}");
}

// ---- 18. Pagination: disabled and busy -------------------------------------

#[test]
fn item_18_pagination_disabled_and_busy() {
    let offset = RwSignal::new(20_usize);
    let limit = RwSignal::new(20_usize);
    let h = html! { <Pagination offset limit total=100_usize page_sizes=vec![20, 50] busy=true /> };
    assert!(tag(&h, "cl-pager").contains(r#"aria-busy="true""#), "{h}");
    assert!(h.contains("cl-loader cl-loader--xs"), "{h}");
    assert!(tag(&h, "cl-pager__select").contains("disabled"), "{h}");
    let buttons = h.matches("<button").count();
    let off = h
        .split("<button")
        .skip(1)
        .filter(|b| b[..b.find('>').unwrap()].contains("disabled"))
        .count();
    assert_eq!((buttons, off), (2, 2), "{h}");

    // Not busy: Previous and Next work on page 2 of 5.
    let h = html! { <Pagination offset limit total=100_usize /> };
    let off = h
        .split("<button")
        .skip(1)
        .filter(|b| b[..b.find('>').unwrap()].contains("disabled"))
        .count();
    assert_eq!(off, 0, "{h}");
    let h = html! { <Pagination offset limit total=100_usize disabled=true /> };
    let off = h
        .split("<button")
        .skip(1)
        .filter(|b| b[..b.find('>').unwrap()].contains("disabled"))
        .count();
    assert_eq!(off, 2, "{h}");
}

// ---- 19. A fixed table does not cut a message row --------------------------

#[test]
fn item_19_fixed_table_wraps_a_message_row() {
    let r = rule(".cl-table--fixed td[colspan]");
    assert!(r.contains("white-space: normal"), "{r}");
    let h = html! { <Table fixed=true><tbody><TableEmpty message="No rows." colspan=4 /></tbody></Table> };
    assert!(h.contains(r#"<td colspan="4">No rows.</td>"#), "{h}");
}

// ---- 20. Inputs show focus with a ring -------------------------------------

#[test]
fn item_20_inputs_have_a_focus_ring() {
    assert!(rule(".cl-input:focus-visible").contains("outline: 2px solid var(--ice)"));
}

// ---- 21. AppShell reads no disposed signal after unmount -------------------

#[test]
fn item_21_appshell_frame_callback_tries_its_node() {
    // The frame callback that moves focus into an open drawer can run after
    // the shell is gone; it must read its node with `try_`. A browser frame
    // does not run in a native test, so the source is checked.
    let src = include_str!("../src/frame.rs");
    let raf = &src[src
        .find("request_animation_frame(move ||")
        .expect("frame callback")..];
    let body = &raf[..raf.find("});").unwrap()];
    assert!(body.contains("nav_ref.try_get_untracked()"), "{body}");
    assert!(!body.contains("nav_ref.get_untracked()"), "{body}");
    // And the shell renders and goes away with no panic.
    let _ = html! { <AppShell navbar=Box::new(|| view! { "nav" }.into_any())>"x"</AppShell> };
}

// ---- 22. The missing components --------------------------------------------

#[test]
fn item_22_checkbox() {
    let on = RwSignal::new(true);
    let h = html! { <Checkbox checked=on label="Seed after import" description="Keeps the torrent." id="c" /> };
    let input = tag(&h, "<input");
    assert!(
        input.contains(r#"type="checkbox""#) && input.contains(r#"id="c""#),
        "{input}"
    );
    assert!(input.contains(r#"aria-describedby="c-help""#), "{input}");
    assert!(h.contains("Seed after import"), "{h}");
    assert!(rule(".cl-checkbox:focus-visible").contains("outline"));
}

#[test]
fn item_22_file_input() {
    let h = html! { <FileInput label="Subtitles" accept=".srt,.vtt" multiple=true /> };
    let input = tag(&h, "<input");
    assert!(
        input.contains(r#"type="file""#) && input.contains(r#"accept=".srt,.vtt""#),
        "{input}"
    );
    assert!(input.contains("multiple"), "{input}");
    assert!(
        h.contains("Choose file") && h.contains("No file chosen"),
        "{h}"
    );
    // With no label, the button text names the input.
    let h = html! { <FileInput /> };
    assert!(
        tag(&h, "<input").contains(r#"aria-label="Choose file""#),
        "{h}"
    );
}

#[test]
fn item_22_slider() {
    let v = RwSignal::new(40.0);
    let h = html! {
        <Slider value=v label="Volume" format=Callback::new(|v: f64| format!("{v}%")) />
    };
    let input = tag(&h, "<input");
    assert!(input.contains(r#"type="range""#), "{input}");
    assert!(input.contains(r#"aria-valuetext="40%""#), "{input}");
    assert!(input.contains("--cl-slider-fill:40%"), "{input}");
    assert!(h.contains(">40%</span>"), "{h}");
}

#[test]
fn item_22_disclosure_and_accordion() {
    let h = html! {
        <Accordion>
            <Disclosure title="One">"a"</Disclosure>
            <Disclosure title="Two" default_open=true>"b"</Disclosure>
        </Accordion>
    };
    assert_eq!(h.matches("<details").count(), 2, "{h}");
    assert!(h.contains("<summary"), "{h}");
    let first = tag(&h, "<details");
    let name = first
        .split(r#"name=""#)
        .nth(1)
        .expect("an accordion name")
        .split('"')
        .next()
        .unwrap();
    assert_eq!(h.matches(&format!(r#"name="{name}""#)).count(), 2, "{h}");
    // `multiple`: no name, so any number open.
    let h = html! { <Accordion multiple=true><Disclosure title="One">"a"</Disclosure></Accordion> };
    assert!(!tag(&h, "<details").contains("name="), "{h}");
}

#[test]
fn item_22_skeleton_and_loader() {
    let h = html! { <Skeleton lines=3 /> };
    assert_eq!(h.matches(r#"class="cl-skeleton""#).count(), 3, "{h}");
    assert!(h.contains(r#"aria-hidden="true""#), "{h}");
    let h = html! { <Skeleton circle=true height="32px" /> };
    assert!(h.contains("width:32px;height:32px;"), "{h}");

    let h = html! { <Loader size="xs" /> };
    assert!(
        h.contains(r#"class="cl-loader cl-loader--xs""#) && h.contains(r#"aria-hidden="true""#),
        "{h}"
    );
    let h = html! { <Loader label="Loading runs" /> };
    assert!(
        h.contains(r#"role="status""#) && h.contains("Loading runs"),
        "{h}"
    );
}

#[test]
fn item_22_combobox() {
    let v = RwSignal::new("tv".to_string());
    let opts = vec![
        ("movie".to_string(), "Movie".to_string()),
        ("tv".to_string(), "TV show".to_string()),
    ];
    let h = html! { <Combobox value=v options=opts label="Kind" id="k" /> };
    let input = tag(&h, "<input");
    assert!(input.contains(r#"role="combobox""#), "{input}");
    assert!(input.contains(r#"aria-expanded="false""#), "{input}");
    assert!(input.contains(r#"aria-controls="k-list""#), "{input}");
    assert!(input.contains(r#"aria-autocomplete="list""#), "{input}");
    assert_eq!(
        combobox_matches(&["Movie".into(), "TV show".into()], "sh"),
        vec![1]
    );
}
