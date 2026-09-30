//! Aurora frame — the page chrome and the layers over a page.
//!
//! - [`AppShell`] + [`SideNav`] / [`SideNavGroup`] / [`SideNavLink`]: the app
//!   scaffold. A header, a sidebar with groups of links, a brand slot, a footer
//!   slot. Below 768px the sidebar becomes a drawer behind a menu button.
//! - [`PageHeader`]: back link, `<h1>` title, sub line, meta line, actions.
//! - [`Modal`] and [`Drawer`]: dialogs with sizes, a footer, Escape to close,
//!   and a focus trap that gives focus back to the opener.
//! - [`ConfirmDialog`]: a `Modal` preset for actions that destroy or change
//!   data: consequence text, a list of what changes, a "type the name" check,
//!   a busy state.
//! - [`ToastStack`] + [`Toaster`] ([`provide_toaster`], [`use_toaster`]):
//!   short messages from any part of the app.
//! - [`Tabs`] + [`TabPanel`]: a tab list driven by a signal or by links.
//! - [`Card`]: a panel that a person can click or open with the keyboard.
//!
//! Everything here is also re-exported from [`crate::components`], so
//! `use aurora_leptos::components::*;` finds it.
//!
//! The pure logic (focus-trap steps, tab keys, the toast queue, the type-to-
//! confirm check) is plain Rust with unit tests at the end of this file.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::tokens::{fg_for, fill_for, token};

/// A unique DOM id (for `aria-labelledby`, `aria-controls`, `for`).
fn next_id(prefix: &str) -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("cl-{prefix}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

// ============================================================================
// Pure logic
// ============================================================================

/// What a focus trap does with one Tab key press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapStep {
    /// Let the browser move focus (it stays inside).
    Native,
    /// Keep focus where it is (nothing inside can take focus).
    Stay,
    /// Move focus to the focusable element at this index.
    Focus(usize),
}

/// The focus-trap step for a Tab (or Shift+Tab when `backward`) press.
///
/// `count` is the number of focusable elements inside the trap. `current` is
/// the index of the element that has focus, or `None` when focus is outside
/// the list (on the dialog itself, or lost). Focus wraps from the last element
/// to the first, and from the first to the last.
pub fn trap_step(count: usize, current: Option<usize>, backward: bool) -> TrapStep {
    if count == 0 {
        return TrapStep::Stay;
    }
    let last = count - 1;
    match current {
        None => TrapStep::Focus(if backward { last } else { 0 }),
        Some(0) if backward => TrapStep::Focus(last),
        Some(i) if !backward && i >= last => TrapStep::Focus(0),
        Some(_) => TrapStep::Native,
    }
}

/// The tab that a key moves to in a horizontal tab list, or `None` when the
/// key does nothing.
///
/// ArrowRight / ArrowLeft move to the next / previous tab and wrap. Home / End
/// move to the first / last tab. Disabled tabs are skipped.
pub fn tab_key_target(key: &str, current: usize, disabled: &[bool]) -> Option<usize> {
    let n = disabled.len();
    if n == 0 {
        return None;
    }
    let current = current % n;
    let enabled = |i: &usize| !disabled[*i];
    match key {
        "Home" => (0..n).find(enabled),
        "End" => (0..n).rev().find(enabled),
        "ArrowRight" => (1..=n).map(|s| (current + s) % n).find(enabled),
        "ArrowLeft" => (1..=n).map(|s| (current + n - s) % n).find(enabled),
        _ => None,
    }
}

/// A DOM id for one tab or panel: `{base}-{part}-{value}`, with every
/// character that is not a letter, a digit, `-` or `_` changed to `-`.
pub fn tab_dom_id(base: &str, value: &str, part: &str) -> String {
    let clean: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("{base}-{part}-{clean}")
}

/// True when the typed text lets the person confirm. An empty `expected`
/// means there is no check. Spaces at the ends do not count.
pub fn confirm_matches(typed: &str, expected: &str) -> bool {
    let expected = expected.trim();
    expected.is_empty() || typed.trim() == expected
}

/// The time (ms) left on a toast timer after a pause at `now`, when the timer
/// started at `started` with `remaining` ms left. Never below zero.
pub fn remaining_after_pause(remaining: f64, started: f64, now: f64) -> f64 {
    (remaining - (now - started).max(0.0)).max(0.0)
}

/// The kind of a toast. It sets the colour and the ARIA role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    /// The hue token of this kind.
    pub fn color(self) -> &'static str {
        match self {
            ToastKind::Info => token::ICE,
            ToastKind::Success => token::OK,
            ToastKind::Warning => token::GOLD,
            ToastKind::Error => token::BAD,
        }
    }

    /// `alert` for errors (read at once), `status` for the others.
    pub fn role(self) -> &'static str {
        match self {
            ToastKind::Error => "alert",
            _ => "status",
        }
    }

    fn class(self) -> &'static str {
        match self {
            ToastKind::Info => "info",
            ToastKind::Success => "success",
            ToastKind::Warning => "warning",
            ToastKind::Error => "error",
        }
    }
}

/// One toast in the queue.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
    pub id: u64,
    pub kind: ToastKind,
    pub text: String,
    /// Time on screen in ms. `None`: the `ToastStack` default. `Some(0)`: the
    /// toast stays until the person dismisses it.
    pub timeout_ms: Option<u64>,
}

/// The toasts on screen, oldest first. At most `max`: a new toast pushes out
/// the oldest one.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastQueue {
    items: Vec<ToastItem>,
    next_id: u64,
    max: usize,
}

impl Default for ToastQueue {
    fn default() -> Self {
        Self::new(5)
    }
}

impl ToastQueue {
    /// An empty queue that holds at most `max` toasts (at least 1).
    pub fn new(max: usize) -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
            max: max.max(1),
        }
    }

    /// Adds a toast and returns its id.
    pub fn push(
        &mut self,
        kind: ToastKind,
        text: impl Into<String>,
        timeout_ms: Option<u64>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(ToastItem {
            id,
            kind,
            text: text.into(),
            timeout_ms,
        });
        if self.items.len() > self.max {
            let extra = self.items.len() - self.max;
            self.items.drain(..extra);
        }
        id
    }

    /// Removes the toast with this id. Returns false when it was not there.
    pub fn dismiss(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|t| t.id != id);
        self.items.len() != before
    }

    /// Removes all toasts.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// The toasts, oldest first.
    pub fn items(&self) -> &[ToastItem] {
        &self.items
    }
}

// ============================================================================
// DOM helpers (focus)
// ============================================================================

const FOCUSABLE: &str = "a[href], button:not([disabled]), \
    input:not([disabled]):not([type=\"hidden\"]), select:not([disabled]), \
    textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

/// The elements inside `root` that can take focus, in DOM order. Elements
/// that are not rendered (no layout box) are left out.
fn focusables(root: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    let Ok(list) = root.query_selector_all(FOCUSABLE) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|e| e.get_client_rects().length() > 0)
        .collect()
}

fn first_focusable_in(root: &web_sys::Element, selector: &str) -> Option<web_sys::HtmlElement> {
    let part = root.query_selector(selector).ok().flatten()?;
    focusables(&part).into_iter().next()
}

/// Moves focus into a dialog: first an element marked `data-autofocus` (or
/// `autofocus`), then the first control in the body, then in the footer, then
/// any control, then the dialog itself.
fn focus_initial(root: &web_sys::Element) {
    let marked = root
        .query_selector("[data-autofocus], [autofocus]")
        .ok()
        .flatten()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
    let target = marked
        .or_else(|| first_focusable_in(root, ".cl-modal__body"))
        .or_else(|| first_focusable_in(root, ".cl-modal__footer"))
        .or_else(|| focusables(root).into_iter().next());
    match target {
        Some(el) => {
            let _ = el.focus();
        }
        None => {
            if let Some(el) = root.dyn_ref::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
        }
    }
}

/// Keeps Tab and Shift+Tab inside `root`.
fn trap_tab(root: &web_sys::Element, ev: &web_sys::KeyboardEvent) {
    let items = focusables(root);
    let active = document().active_element();
    let current = active.and_then(|a| {
        items
            .iter()
            .position(|e| e.unchecked_ref::<web_sys::Node>().is_same_node(Some(&a)))
    });
    match trap_step(items.len(), current, ev.shift_key()) {
        TrapStep::Native => {}
        TrapStep::Stay => ev.prevent_default(),
        TrapStep::Focus(i) => {
            ev.prevent_default();
            let _ = items[i].focus();
        }
    }
}

fn active_html_element() -> Option<web_sys::HtmlElement> {
    document()
        .active_element()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
}

// ============================================================================
// AppShell + SideNav
// ============================================================================

/// The app scaffold: a header across the top, a sidebar, and the main area.
///
/// - `header` (optional): the top bar content (search, user menu, theme toggle).
/// - `navbar` (optional): the sidebar content, usually a [`SideNav`]. Leave it
///   out for a top-bar-only app.
/// - `brand` (optional): the product mark and name. It shows in the header;
///   with no `header` it shows at the top of the sidebar.
/// - `contained`: draw the shell as a bordered panel of fixed height (for docs
///   and galleries). By default the shell fills the page.
/// - `menu_label`: the accessible name of the menu button (default "Menu").
///
/// Below 768px the sidebar leaves the layout. A menu button in the header
/// opens it as a drawer from the left. Escape, a click on the scrim, or a click
/// on a link closes it, and focus goes back to the menu button.
#[component]
pub fn AppShell(
    #[prop(optional)] header: Option<Children>,
    #[prop(optional)] navbar: Option<Children>,
    #[prop(optional)] brand: Option<ChildrenFn>,
    #[prop(optional)] contained: bool,
    #[prop(default = "Menu".to_string(), into)] menu_label: String,
    children: Children,
) -> impl IntoView {
    let nav_open = RwSignal::new(false);
    let has_nav = navbar.is_some();
    let has_header = header.is_some();
    let nav_id = next_id("appshell-nav");
    let menu_ref = NodeRef::<html::Button>::new();
    let nav_ref = NodeRef::<html::Div>::new();

    let mut class = String::from("cl-appshell");
    if contained {
        class.push_str(" cl-appshell--contained");
    }
    if !has_nav {
        class.push_str(" cl-appshell--no-nav");
    }
    if !has_header {
        class.push_str(" cl-appshell--no-header");
    }

    let close_nav = move |refocus: bool| {
        if nav_open.get_untracked() {
            nav_open.set(false);
            if refocus {
                if let Some(b) = menu_ref.get_untracked() {
                    let _ = b.focus();
                }
            }
        }
    };

    // When the drawer opens, move focus to its first link.
    Effect::new(move |_| {
        if nav_open.get() {
            request_animation_frame(move || {
                if let Some(nav) = nav_ref.get_untracked() {
                    if let Some(first) = focusables(&nav).into_iter().next() {
                        let _ = first.focus();
                    }
                }
            });
        }
    });

    let show_bar = has_header || has_nav || brand.is_some();
    let bar_brand = brand
        .clone()
        .map(|b| view! { <div class="cl-appshell__brand">{b()}</div> });
    let side_brand = if has_header {
        None
    } else {
        brand
            .map(|b| view! { <div class="cl-appshell__brand cl-appshell__brand--side">{b()}</div> })
    };
    let menu_button = has_nav.then(|| {
        let nav_id = nav_id.clone();
        view! {
            <button
                node_ref=menu_ref
                type="button"
                class="cl-appshell__menu"
                aria-label=menu_label
                aria-controls=nav_id
                aria-expanded=move || if nav_open.get() { "true" } else { "false" }
                on:click=move |_| nav_open.update(|v| *v = !*v)
            >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                    stroke-width="2" stroke-linecap="round" aria-hidden="true">
                    <path d="M4 6h16M4 12h16M4 18h16" />
                </svg>
            </button>
        }
    });
    let bar = show_bar.then(|| {
        view! {
            <header class="cl-appshell__header">
                {menu_button}
                {bar_brand}
                {header.map(|h| view! { <div class="cl-appshell__header-content">{h()}</div> })}
            </header>
        }
    });
    let sidebar = navbar.map(|n| {
        view! {
            <div class="cl-appshell__scrim" on:click=move |_| close_nav(true)></div>
            <div
                class="cl-appshell__navbar"
                id=nav_id
                node_ref=nav_ref
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if !nav_open.get_untracked() {
                        return;
                    }
                    match ev.key().as_str() {
                        "Escape" => {
                            ev.prevent_default();
                            close_nav(true);
                        }
                        "Tab" => {
                            if let Some(nav) = nav_ref.get_untracked() {
                                trap_tab(&nav, &ev);
                            }
                        }
                        _ => {}
                    }
                }
                on:click=move |ev: web_sys::MouseEvent| {
                    // A click on a link inside the drawer closes it.
                    let on_link = ev
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .and_then(|e| e.closest("a[href]").ok().flatten())
                        .is_some();
                    if on_link {
                        close_nav(false);
                    }
                }
            >
                {side_brand}
                {n()}
            </div>
        }
    });

    view! {
        <div class=class class:cl-appshell--nav-open=move || nav_open.get()>
            {bar}
            {sidebar}
            <main class="cl-appshell__main">{children()}</main>
        </div>
    }
}

/// The sidebar navigation: a `<nav>` landmark with groups of links and an
/// optional footer (scope switcher, user, sign-out) pinned to the bottom.
///
/// Put [`SideNavGroup`]s and [`SideNavLink`]s inside. `label` is the name of
/// the landmark (default "Main").
#[component]
pub fn SideNav(
    #[prop(default = "Main".to_string(), into)] label: String,
    #[prop(optional)] footer: Option<Children>,
    children: Children,
) -> impl IntoView {
    view! {
        <nav class="cl-sidenav" aria-label=label>
            <div class="cl-sidenav__links">{children()}</div>
            {footer.map(|f| view! { <div class="cl-sidenav__footer">{f()}</div> })}
        </nav>
    }
}

/// A group of [`SideNavLink`]s with an optional label ("Orchestration",
/// "System"). With a label, the group is an ARIA `group` named by it.
#[component]
pub fn SideNavGroup(#[prop(optional, into)] label: String, children: Children) -> impl IntoView {
    if label.is_empty() {
        view! { <div class="cl-sidenav__group">{children()}</div> }.into_any()
    } else {
        let id = next_id("navgroup");
        let labelled_by = id.clone();
        view! {
            <div class="cl-sidenav__group" role="group" aria-labelledby=labelled_by>
                <div class="cl-sidenav__label" id=id>{label}</div>
                {children()}
            </div>
        }
        .into_any()
    }
}

/// One link in a [`SideNav`]: a plain `<a href>`, so it works with or without
/// a router (leptos_router handles clicks on plain anchors).
///
/// - `active`: a bool, a signal or a closure (`move || path.get() == "/runs"`).
///   The active link gets `aria-current="page"`.
/// - `count` (optional): a badge after the label. `count_color` sets its hue
///   (a token, e.g. `token::BAD`); by default it is neutral.
/// - `marker` (optional): a small coloured square before the label (a token).
/// - `icon` (optional): an icon before the label.
/// - `on_click` (optional): runs on click (for apps that route with a signal).
#[component]
pub fn SideNavLink(
    #[prop(into)] href: String,
    #[prop(optional, into)] active: Signal<bool>,
    #[prop(optional, into)] count: MaybeProp<usize>,
    #[prop(optional, into)] count_color: String,
    #[prop(optional, into)] marker: String,
    #[prop(optional)] icon: Option<Children>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let count_style = if count_color.is_empty() {
        String::new()
    } else {
        format!(
            "background:{};color:{};",
            fill_for(&count_color),
            fg_for(&count_color)
        )
    };
    let marker_view = (!marker.is_empty()).then(|| {
        view! {
            <span class="cl-navlink__marker" style=format!("background:{marker};") aria-hidden="true"></span>
        }
    });
    view! {
        <a
            class="cl-navlink"
            class:cl-navlink--active=move || active.get()
            href=href
            aria-current=move || active.get().then_some("page")
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {marker_view}
            {icon.map(|i| view! { <span class="cl-navlink__icon" aria-hidden="true">{i()}</span> })}
            <span class="cl-navlink__label">{children()}</span>
            {move || count.get().map(|n| {
                view! { <span class="cl-navlink__count" style=count_style.clone()>{n}</span> }
            })}
        </a>
    }
}

// ============================================================================
// PageHeader
// ============================================================================

/// The page title block.
///
/// - `title`: the page name, in an `<h1>`.
/// - `sub` (optional): a mono line under the title (an id, a path, a count).
/// - `back_href` + `back_label` (optional): a "← Executions" link above the
///   title. The label defaults to "Back".
/// - `meta` (optional): a line of pills or facts under the title.
/// - `actions` (optional): buttons on the right. `right` is the older name of
///   the same slot and still works; when both are given, both show.
#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(optional, into)] sub: String,
    #[prop(optional)] right: Option<Children>,
    #[prop(optional, into)] back_href: String,
    #[prop(optional, into)] back_label: String,
    #[prop(optional)] meta: Option<Children>,
    #[prop(optional)] actions: Option<Children>,
) -> impl IntoView {
    let has_sub = !sub.is_empty();
    let back = (!back_href.is_empty()).then(|| {
        let label = if back_label.is_empty() {
            "Back".to_string()
        } else {
            back_label
        };
        view! {
            <a class="cl-page-header__back" href=back_href>
                <span aria-hidden="true">"← "</span>
                {label}
            </a>
        }
    });
    let has_actions = right.is_some() || actions.is_some();
    view! {
        <div class="cl-page-header">
            <div class="cl-page-header__main">
                {back}
                // A real heading element: the page title carries the ARIA
                // heading role for assistive tech and role-based selectors.
                <h1 class="cl-page-header__title">{title}</h1>
                {has_sub.then(|| view! { <div class="cl-page-header__sub">{sub}</div> })}
                {meta.map(|m| view! { <div class="cl-page-header__meta">{m()}</div> })}
            </div>
            {has_actions.then(|| view! {
                <div class="cl-page-header__actions">
                    {right.map(|r| r())}
                    {actions.map(|a| a())}
                </div>
            })}
        </div>
    }
}

// ============================================================================
// Modal + Drawer (one dialog frame)
// ============================================================================

struct DialogProps {
    /// "modal" or "drawer": the CSS block name.
    kind: &'static str,
    open: RwSignal<bool>,
    title: String,
    size: String,
    children: ChildrenFn,
    footer: Option<ChildrenFn>,
    close_on_scrim: bool,
    locked: Signal<bool>,
    on_close: Option<Callback<()>>,
}

fn dialog_frame(p: DialogProps) -> impl IntoView {
    let DialogProps {
        kind,
        open,
        title,
        size,
        children,
        footer,
        close_on_scrim,
        locked,
        on_close,
    } = p;
    let size = match size.as_str() {
        "sm" | "md" | "lg" | "xl" => size,
        _ => "md".to_string(),
    };
    // The element that had focus before the dialog opened.
    let restore = StoredValue::new_local(None::<web_sys::HtmlElement>);
    let scrim_down = StoredValue::new(false);

    let close = move || {
        if locked.get_untracked() {
            return;
        }
        open.set(false);
        if let Some(cb) = on_close {
            cb.run(());
        }
    };

    // Give focus back to the opener when the dialog closes.
    Effect::new(move |was_open: Option<bool>| {
        let is_open = open.get();
        if !is_open && was_open == Some(true) {
            if let Some(el) = restore.try_update_value(|v| v.take()).flatten() {
                if el.is_connected() {
                    let _ = el.focus();
                }
            }
        }
        is_open
    });

    view! {
        {move || open.get().then(|| {
            restore.set_value(active_html_element());
            let panel = NodeRef::<html::Div>::new();
            Effect::new(move |_| {
                if let Some(el) = panel.get() {
                    focus_initial(&el);
                }
            });
            let title = title.clone();
            let aria_title = title.clone();
            let children = children.clone();
            let footer = footer.clone();
            view! {
                <div
                    class=format!("cl-{kind}-overlay")
                    on:mousedown=move |ev: web_sys::MouseEvent| {
                        let on_scrim = ev.target() == ev.current_target();
                        scrim_down.set_value(on_scrim);
                        if on_scrim {
                            // Keep focus in the dialog: a press on the scrim
                            // would move it to <body>, out of reach of Escape.
                            ev.prevent_default();
                        }
                    }
                    on:click=move |ev: web_sys::MouseEvent| {
                        let on_scrim = scrim_down.get_value() && ev.target() == ev.current_target();
                        scrim_down.set_value(false);
                        if on_scrim && close_on_scrim {
                            close();
                        }
                    }
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        match ev.key().as_str() {
                            "Escape" => {
                                ev.prevent_default();
                                ev.stop_propagation();
                                close();
                            }
                            "Tab" => {
                                ev.stop_propagation();
                                if let Some(el) = panel.get_untracked() {
                                    trap_tab(&el, &ev);
                                }
                            }
                            _ => {}
                        }
                    }
                >
                    // ARIA dialog semantics: assistive tech and role-based
                    // selectors resolve the dialog by its title.
                    <div
                        node_ref=panel
                        class=format!("cl-{kind} cl-{kind}--{size}")
                        role="dialog"
                        aria-modal="true"
                        aria-label=aria_title
                        tabindex="-1"
                    >
                        <div class="cl-modal__header">
                            <span class="cl-modal__title">{title}</span>
                            <button
                                type="button"
                                class="cl-modal__close"
                                aria-label="Close"
                                disabled=move || locked.get()
                                on:click=move |_| close()
                            >
                                "×"
                            </button>
                        </div>
                        <div class="cl-modal__body">{children()}</div>
                        {footer.map(|f| view! { <div class="cl-modal__footer">{f()}</div> })}
                    </div>
                </div>
            }
        })}
    }
}

/// A dialog over the page, open while `open` is true.
///
/// - `title`: shown in the header; also the accessible name of the dialog.
/// - `size`: `"sm"` (400px), `"md"` (520px, default), `"lg"` (760px) or
///   `"xl"` (1060px). On a small screen the dialog is the screen width less a
///   16px margin.
/// - `footer` (optional): buttons in a bar under the body, on the right.
/// - `close_on_scrim` (default true): a click on the dim area closes it.
/// - `locked` (optional): while true, Escape, the scrim and × do not close it
///   (use it while a request runs).
/// - `on_close` (optional): runs when the person closes the dialog with
///   Escape, the scrim or ×.
///
/// On open, focus moves into the dialog: to an element with
/// `data-autofocus`, else the first control in the body, else the footer.
/// Tab and Shift+Tab stay inside. On close, focus goes back to the element
/// that had it before.
#[component]
pub fn Modal(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] size: String,
    #[prop(optional)] footer: Option<ChildrenFn>,
    #[prop(default = true)] close_on_scrim: bool,
    #[prop(optional, into)] locked: Signal<bool>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    dialog_frame(DialogProps {
        kind: "modal",
        open,
        title,
        size,
        children,
        footer,
        close_on_scrim,
        locked,
        on_close,
    })
}

/// A panel that slides in from the right, over a scrim (a detail view).
///
/// Same props and keyboard behaviour as [`Modal`]. `size`: `"sm"` (360px),
/// `"md"` (430px, default), `"lg"` (640px) or `"xl"` (860px); never wider than
/// 92% of the screen.
#[component]
pub fn Drawer(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] size: String,
    #[prop(optional)] footer: Option<ChildrenFn>,
    #[prop(default = true)] close_on_scrim: bool,
    #[prop(optional, into)] locked: Signal<bool>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    dialog_frame(DialogProps {
        kind: "drawer",
        open,
        title,
        size,
        children,
        footer,
        close_on_scrim,
        locked,
        on_close,
    })
}

// ============================================================================
// ConfirmDialog
// ============================================================================

/// A confirmation for an action that destroys or changes data.
///
/// - `title`: the question ("Delete workflow?").
/// - `message` (optional): what happens, in one or two sentences.
/// - `impacts` (optional): a list of what the action also changes (for
///   example the items an archive cascades to), under `impacts_label`.
/// - `confirm_text` (optional): the person must type this text (a name) to
///   enable the confirm button.
/// - `confirm_label` / `cancel_label`: button text ("Confirm" / "Cancel").
/// - `danger` (default true): the confirm button is red.
/// - `busy` (optional): while true, the buttons are disabled, the confirm
///   button shows a spinner, and the dialog does not close.
/// - `on_confirm`: runs when the person confirms. The dialog stays open: set
///   `open` to false when the work is done (or at once).
/// - `on_cancel` (optional): runs on Cancel, Escape, the scrim or ×. The
///   dialog closes itself.
/// - children (optional): more content under the list (an error, a preview).
#[component]
pub fn ConfirmDialog(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] message: String,
    #[prop(optional, into)] impacts: Signal<Vec<String>>,
    #[prop(default = "This also changes:".to_string(), into)] impacts_label: String,
    #[prop(optional, into)] confirm_text: String,
    #[prop(default = "Confirm".to_string(), into)] confirm_label: String,
    #[prop(default = "Cancel".to_string(), into)] cancel_label: String,
    #[prop(default = true)] danger: bool,
    #[prop(optional, into)] busy: Signal<bool>,
    #[prop(default = "sm".to_string(), into)] size: String,
    on_confirm: Callback<()>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let typed = RwSignal::new(String::new());
    // Each time the dialog opens, the check starts empty.
    Effect::new(move |_| {
        if open.get() {
            typed.set(String::new());
        }
    });
    let expected = StoredValue::new(confirm_text.clone());
    let has_check = !confirm_text.trim().is_empty();
    let can_confirm = Memo::new(move |_| {
        !busy.get() && expected.with_value(|e| confirm_matches(&typed.get(), e))
    });
    let confirm = move || {
        if can_confirm.get_untracked() {
            on_confirm.run(());
        }
    };
    let cancel = move || {
        if busy.get_untracked() {
            return;
        }
        open.set(false);
        if let Some(cb) = on_cancel {
            cb.run(());
        }
    };

    let body: ChildrenFn = {
        let message = message.clone();
        let impacts_label = impacts_label.clone();
        let children = children.clone();
        std::sync::Arc::new(move || {
            let input_id = next_id("confirm");
            let message = message.clone();
            let impacts_label = impacts_label.clone();
            let children = children.clone();
            view! {
                <div class="cl-confirm">
                    {(!message.is_empty()).then(|| view! { <p class="cl-confirm__message">{message}</p> })}
                    {move || {
                        let list = impacts.get();
                        (!list.is_empty()).then(|| view! {
                            <div class="cl-confirm__impacts">
                                <div class="cl-confirm__impacts-label">{impacts_label.clone()}</div>
                                <ul class="cl-confirm__list">
                                    {list.into_iter().map(|i| view! { <li>{i}</li> }).collect_view()}
                                </ul>
                            </div>
                        })
                    }}
                    {has_check.then(|| {
                        let name = expected.get_value();
                        view! {
                            <div class="cl-field cl-confirm__check">
                                <label class="cl-field__label" for=input_id.clone()>
                                    "Type " <code class="cl-code">{name}</code> " to confirm"
                                </label>
                                <input
                                    class="cl-input cl-mono"
                                    id=input_id
                                    type="text"
                                    autocomplete="off"
                                    spellcheck="false"
                                    data-autofocus=""
                                    disabled=move || busy.get()
                                    prop:value=move || typed.get()
                                    on:input=move |e| typed.set(event_target_value(&e))
                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            ev.prevent_default();
                                            confirm();
                                        }
                                    }
                                />
                            </div>
                        }
                    })}
                    {children.map(|c| c())}
                </div>
            }
            .into_any()
        })
    };

    let footer: ChildrenFn = {
        let confirm_label = confirm_label.clone();
        let cancel_label = cancel_label.clone();
        std::sync::Arc::new(move || {
            let confirm_label = confirm_label.clone();
            let cancel_label = cancel_label.clone();
            let confirm_class = if danger {
                "cl-btn cl-btn--filled cl-btn--bad"
            } else {
                "cl-btn cl-btn--filled"
            };
            view! {
                <button
                    type="button"
                    class="cl-btn cl-btn--default"
                    data-autofocus=(!has_check).then_some("")
                    disabled=move || busy.get()
                    on:click=move |_| cancel()
                >
                    {cancel_label}
                </button>
                <button
                    type="button"
                    class=confirm_class
                    disabled=move || !can_confirm.get()
                    aria-busy=move || if busy.get() { "true" } else { "false" }
                    on:click=move |_| confirm()
                >
                    {move || busy.get().then(|| view! { <span class="cl-btn__spinner" aria-hidden="true"></span> })}
                    {confirm_label}
                </button>
            }
            .into_any()
        })
    };

    dialog_frame(DialogProps {
        kind: "modal",
        open,
        title,
        size,
        children: body,
        footer: Some(footer),
        close_on_scrim: true,
        locked: busy,
        on_close: on_cancel,
    })
}

// ============================================================================
// Toasts
// ============================================================================

/// A handle to the toast queue. `Copy`: capture it once and call it from event
/// handlers and async tasks.
///
/// ```ignore
/// let toaster = use_toaster();
/// toaster.success("Workflow saved");
/// toaster.toast(ToastKind::Error, "The server did not answer");
/// ```
#[derive(Clone, Copy)]
pub struct Toaster {
    queue: RwSignal<ToastQueue>,
}

impl Default for Toaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Toaster {
    /// A new, empty queue (at most 5 toasts on screen).
    pub fn new() -> Self {
        Self {
            queue: RwSignal::new(ToastQueue::default()),
        }
    }

    /// Shows a toast for the default time. Returns its id.
    pub fn toast(&self, kind: ToastKind, text: impl Into<String>) -> u64 {
        let text = text.into();
        let mut id = 0;
        self.queue.update(|q| id = q.push(kind, text, None));
        id
    }

    /// Shows a toast for `ms` milliseconds; `0` keeps it until dismissed.
    pub fn toast_for(&self, kind: ToastKind, text: impl Into<String>, ms: u64) -> u64 {
        let text = text.into();
        let mut id = 0;
        self.queue.update(|q| id = q.push(kind, text, Some(ms)));
        id
    }

    pub fn info(&self, text: impl Into<String>) -> u64 {
        self.toast(ToastKind::Info, text)
    }

    pub fn success(&self, text: impl Into<String>) -> u64 {
        self.toast(ToastKind::Success, text)
    }

    pub fn warning(&self, text: impl Into<String>) -> u64 {
        self.toast(ToastKind::Warning, text)
    }

    pub fn error(&self, text: impl Into<String>) -> u64 {
        self.toast(ToastKind::Error, text)
    }

    /// Removes one toast.
    pub fn dismiss(&self, id: u64) {
        self.queue.update(|q| {
            q.dismiss(id);
        });
    }

    /// Removes all toasts.
    pub fn clear(&self) {
        self.queue.update(|q| q.clear());
    }

    /// The toasts on screen (tracked), oldest first.
    pub fn items(&self) -> Vec<ToastItem> {
        self.queue.with(|q| q.items().to_vec())
    }
}

/// Creates the toast queue and puts it in the context. Call once at the app
/// root, then mount one [`ToastStack`].
pub fn provide_toaster() -> Toaster {
    let t = Toaster::new();
    provide_context(t);
    t
}

/// The toast queue from the context.
///
/// # Panics
/// When no [`provide_toaster`] (or [`ToastStack`]) is above in the tree.
pub fn use_toaster() -> Toaster {
    use_context::<Toaster>()
        .expect("use_toaster: call provide_toaster() at the app root (or mount a ToastStack)")
}

/// Shows a toast through the context queue. Does nothing when there is no
/// queue. In async code, capture [`use_toaster`] first instead.
pub fn toast(kind: ToastKind, text: impl Into<String>) {
    if let Some(t) = use_context::<Toaster>() {
        t.toast(kind, text);
    }
}

/// The stack of toasts at the bottom right. Mount it once, near the app root.
///
/// - `duration_ms` (default 5000): time on screen of a toast. The timers stop
///   while the pointer is over the stack or focus is in it.
/// - `toaster` (optional): the queue to show. By default the one in the
///   context; if there is none, the stack makes one and provides it.
///
/// A click on a toast dismisses it. Errors have `role="alert"`, the others
/// `role="status"`. Motion stops when the OS asks for reduced motion.
#[component]
pub fn ToastStack(
    #[prop(default = 5000)] duration_ms: u64,
    #[prop(optional)] toaster: Option<Toaster>,
) -> impl IntoView {
    let toaster = toaster
        .or_else(use_context::<Toaster>)
        .unwrap_or_else(provide_toaster);
    let hover = RwSignal::new(false);
    let focus_in = RwSignal::new(false);
    let paused = Signal::derive(move || hover.get() || focus_in.get());
    view! {
        <section
            class="cl-toasts"
            aria-label="Notifications"
            on:mouseenter=move |_| hover.set(true)
            on:mouseleave=move |_| hover.set(false)
            on:focusin=move |_| focus_in.set(true)
            on:focusout=move |_| focus_in.set(false)
        >
            <For each=move || toaster.items() key=|t| t.id let:item>
                <ToastView item=item toaster=toaster paused=paused default_ms=duration_ms />
            </For>
        </section>
    }
}

#[component]
fn ToastView(
    item: ToastItem,
    toaster: Toaster,
    paused: Signal<bool>,
    default_ms: u64,
) -> impl IntoView {
    let id = item.id;
    let ms = item.timeout_ms.unwrap_or(default_ms);
    if ms > 0 {
        let remaining = StoredValue::new(ms as f64);
        let started = StoredValue::new(0.0_f64);
        let handle = StoredValue::new(None::<TimeoutHandle>);
        Effect::new(move |_| {
            if paused.get() {
                if let Some(h) = handle.get_value() {
                    h.clear();
                    handle.set_value(None);
                    remaining.set_value(remaining_after_pause(
                        remaining.get_value(),
                        started.get_value(),
                        js_sys::Date::now(),
                    ));
                }
            } else if handle.get_value().is_none() {
                started.set_value(js_sys::Date::now());
                let wait = remaining.get_value().max(0.0) as u64;
                let h = set_timeout_with_handle(
                    move || toaster.dismiss(id),
                    Duration::from_millis(wait),
                );
                handle.set_value(h.ok());
            }
        });
        on_cleanup(move || {
            if let Some(Some(h)) = handle.try_get_value() {
                h.clear();
            }
        });
    }
    let style = format!("--toast-color:{};", item.kind.color());
    view! {
        <div
            class=format!("cl-toast cl-toast--{}", item.kind.class())
            role=item.kind.role()
            style=style
            on:click=move |_| toaster.dismiss(id)
        >
            <span class="cl-toast__dot" aria-hidden="true"></span>
            <span class="cl-toast__text">{item.text}</span>
            <button
                type="button"
                class="cl-toast__close"
                aria-label="Dismiss"
                on:click=move |ev| {
                    ev.stop_propagation();
                    toaster.dismiss(id);
                }
            >
                "×"
            </button>
        </div>
    }
}

// ============================================================================
// Tabs
// ============================================================================

/// One tab in a [`Tabs`] list.
#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    /// The value the `Tabs` signal holds when this tab is selected.
    pub value: String,
    pub label: String,
    /// With an `href`, the tab is a link (route tabs).
    pub href: Option<String>,
    /// A count after the label.
    pub count: Option<usize>,
    pub disabled: bool,
}

impl TabItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            href: None,
            count: None,
            disabled: false,
        }
    }

    /// Makes the tab a link to `href`.
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn count(mut self, n: usize) -> Self {
        self.count = Some(n);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Copy)]
struct TabsCtx {
    base: StoredValue<String>,
    value: RwSignal<String>,
}

/// A tab list with an underline, and the panels under it.
///
/// - `tabs`: the tabs ([`TabItem`]).
/// - `value`: the selected tab's value. A click or an arrow key sets it.
/// - `label` (optional): the accessible name of the tab list.
/// - children (optional): [`TabPanel`]s. Leave them out for route tabs, where
///   the page under the tabs is the router outlet.
///
/// Signal tabs are buttons: ArrowLeft / ArrowRight / Home / End move focus and
/// select. Route tabs (a [`TabItem::href`]) are links: the arrow keys move
/// focus, and Enter opens the link; set `value` from the route.
#[component]
pub fn Tabs(
    tabs: Vec<TabItem>,
    value: RwSignal<String>,
    #[prop(optional, into)] label: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base = next_id("tabs");
    provide_context(TabsCtx {
        base: StoredValue::new(base.clone()),
        value,
    });
    let has_panels = children.is_some();
    let disabled: Vec<bool> = tabs.iter().map(|t| t.disabled).collect();
    let values: Vec<String> = tabs.iter().map(|t| t.value.clone()).collect();
    let routed: Vec<bool> = tabs.iter().map(|t| t.href.is_some()).collect();
    let first_enabled = disabled.iter().position(|d| !d).unwrap_or(0);
    let meta = StoredValue::new((disabled, values, routed));
    // When no tab matches the value, the first tab takes the Tab stop.
    let any_selected = Memo::new(move |_| {
        let v = value.get();
        meta.with_value(|(_, vals, _)| vals.contains(&v))
    });

    let on_key = move |ev: web_sys::KeyboardEvent, index: usize| {
        let key = ev.key();
        if key == " " && meta.with_value(|(_, _, r)| r[index]) {
            // Space opens a link tab, like a button.
            ev.prevent_default();
            if let Some(el) = ev
                .current_target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
            {
                el.click();
            }
            return;
        }
        let Some(target) = meta.with_value(|(d, _, _)| tab_key_target(&key, index, d)) else {
            return;
        };
        ev.prevent_default();
        let list = ev
            .current_target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|e| e.parent_element());
        if let Some(list) = list {
            if let Ok(nodes) = list.query_selector_all("[role=\"tab\"]") {
                if let Some(el) = nodes
                    .item(target as u32)
                    .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                {
                    let _ = el.focus();
                }
            }
        }
        let (v, is_link) = meta.with_value(|(_, vals, r)| (vals[target].clone(), r[target]));
        if !is_link {
            value.set(v);
        }
    };

    let items = tabs
        .into_iter()
        .enumerate()
        .map(|(i, tab)| {
            let tab_id = tab_dom_id(&base, &tab.value, "tab");
            let panel_id = has_panels.then(|| tab_dom_id(&base, &tab.value, "panel"));
            let v = tab.value.clone();
            let selected = Memo::new(move |_| value.get() == v);
            let takes_stop = move || selected.get() || (!any_selected.get() && i == first_enabled);
            let tabindex = move || if takes_stop() { "0" } else { "-1" };
            let aria_selected = move || if selected.get() { "true" } else { "false" };
            let count = tab
                .count
                .map(|n| view! { <span class="cl-tabs__count">{n}</span> });
            let v_click = tab.value.clone();
            let is_disabled = tab.disabled;
            match tab.href {
                Some(href) => view! {
                    <a
                        class="cl-tabs__tab"
                        class:cl-tabs__tab--active=move || selected.get()
                        role="tab"
                        id=tab_id
                        href=href
                        aria-selected=aria_selected
                        aria-current=move || selected.get().then_some("page")
                        aria-controls=panel_id
                        aria-disabled=is_disabled.then_some("true")
                        tabindex=tabindex
                        on:click=move |ev: web_sys::MouseEvent| {
                            if is_disabled {
                                ev.prevent_default();
                                return;
                            }
                            value.set(v_click.clone());
                        }
                        on:keydown=move |ev| on_key(ev, i)
                    >
                        <span>{tab.label}</span>
                        {count}
                    </a>
                }
                .into_any(),
                None => view! {
                    <button
                        type="button"
                        class="cl-tabs__tab"
                        class:cl-tabs__tab--active=move || selected.get()
                        role="tab"
                        id=tab_id
                        aria-selected=aria_selected
                        aria-controls=panel_id
                        tabindex=tabindex
                        disabled=is_disabled
                        on:click=move |_| value.set(v_click.clone())
                        on:keydown=move |ev| on_key(ev, i)
                    >
                        <span>{tab.label}</span>
                        {count}
                    </button>
                }
                .into_any(),
            }
        })
        .collect_view();
    let aria_label = (!label.is_empty()).then_some(label);
    view! {
        <div class="cl-tabs">
            <div class="cl-tabs__list" role="tablist" aria-label=aria_label>{items}</div>
            {children.map(|c| c())}
        </div>
    }
}

/// The content of one tab. Put it inside [`Tabs`]; `value` matches a
/// [`TabItem::value`]. Only the selected panel renders its children.
#[component]
pub fn TabPanel(#[prop(into)] value: String, children: ChildrenFn) -> impl IntoView {
    let ctx = use_context::<TabsCtx>().expect("TabPanel must be inside Tabs");
    let base = ctx.base.get_value();
    let id = tab_dom_id(&base, &value, "panel");
    let labelled = tab_dom_id(&base, &value, "tab");
    let selected = Memo::new(move |_| ctx.value.get() == value);
    view! {
        <div
            class="cl-tabs__panel"
            role="tabpanel"
            id=id
            aria-labelledby=labelled
            tabindex="0"
            hidden=move || !selected.get()
        >
            {move || selected.get().then(|| children())}
        </div>
    }
}

// ============================================================================
// Card
// ============================================================================

/// A panel with no header, for grids of things (boards, flows, agents).
///
/// - `href` (optional): the card is a link (`<a>`).
/// - `on_click` (optional): the card is a button (`role="button"`, focusable,
///   Enter and Space run it). With `href` too, it runs on click of the link.
/// - `title` (optional): a bold first line.
/// - `label` (optional): the accessible name, when the content is not a good
///   one.
/// - `selected` (optional): draws the card as selected.
///
/// With neither `href` nor `on_click`, the card is a static panel. An
/// interactive card has hover and focus states.
#[component]
pub fn Card(
    #[prop(optional, into)] href: String,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] selected: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let title_view =
        (!title.is_empty()).then(|| view! { <div class="cl-card__title">{title}</div> });
    let aria_label = (!label.is_empty()).then_some(label);
    if !href.is_empty() {
        view! {
            <a
                class="cl-card cl-card--interactive"
                class:cl-card--selected=move || selected.get()
                href=href
                aria-label=aria_label
                on:click=move |_| {
                    if let Some(cb) = on_click {
                        cb.run(());
                    }
                }
            >
                {title_view}
                {children()}
            </a>
        }
        .into_any()
    } else if let Some(cb) = on_click {
        view! {
            <div
                class="cl-card cl-card--interactive"
                class:cl-card--selected=move || selected.get()
                role="button"
                tabindex="0"
                aria-label=aria_label
                aria-pressed=move || selected.get().then_some("true")
                on:click=move |_| cb.run(())
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    // Only when the card itself has focus, not a control in it.
                    if ev.target() != ev.current_target() {
                        return;
                    }
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        cb.run(());
                    }
                }
            >
                {title_view}
                {children()}
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="cl-card" class:cl-card--selected=move || selected.get()>
                {title_view}
                {children()}
            </div>
        }
        .into_any()
    }
}

// ============================================================================
// SecretReveal, CenterScreen, AuthCard
// ============================================================================

/// Shows a secret one time (a new API key, a token, a recovery code): a
/// warning line, the secret in a mono block with a copy button, and an "I
/// saved it" button.
///
/// Put it in a [`Modal`] with `close_on_scrim=false`, and clear the secret
/// in `on_done`, so the secret is not in memory after the person closes it.
///
/// - `secret`: the text to show.
/// - `label` (optional): what the secret is ("API key"), above the block.
/// - `warning` (default: "Copy this now. You cannot see it again after you
///   close this."): the line at the top.
/// - `done_label` (default "I saved it") and `on_done`: the close button.
#[component]
pub fn SecretReveal(
    #[prop(into)] secret: Signal<String>,
    #[prop(optional, into)] label: String,
    #[prop(
        default = "Copy this now. You cannot see it again after you close this.".to_string(),
        into
    )]
    warning: String,
    #[prop(default = "I saved it".to_string(), into)] done_label: String,
    #[prop(optional)] on_done: Option<Callback<()>>,
) -> impl IntoView {
    let label_id = next_id("secret");
    let has_label = !label.is_empty();
    view! {
        <div class="cl-secret">
            <div class="cl-secret__warning" role="note">
                <crate::icons::IconAlert size=16 />
                <span>{warning}</span>
            </div>
            {has_label.then(|| view! { <div class="cl-secret__label" id=label_id.clone()>{label}</div> })}
            <div class="cl-secret__box">
                <code
                    class="cl-secret__value"
                    aria-labelledby=has_label.then(|| label_id.clone())
                >
                    {move || secret.get()}
                </code>
                <crate::components::CopyButton value=secret icon=true label="Copy secret" />
            </div>
            <div class="cl-secret__actions">
                <crate::components::Button
                    on_click=Callback::new(move |_| {
                        if let Some(cb) = on_done {
                            cb.run(());
                        }
                    })
                >
                    {done_label}
                </crate::components::Button>
            </div>
        </div>
    }
}

/// A full-screen area with its content in the centre, on the page
/// background: for sign-in, the OAuth callback, and "no access" or "loading
/// the session" states. Put an [`AuthCard`] in it.
///
/// `contained`: a fixed height (360px) in place of the full screen, for a
/// gallery or a docs page.
#[component]
pub fn CenterScreen(#[prop(optional)] contained: bool, children: Children) -> impl IntoView {
    view! {
        <div class="cl-center-screen" class:cl-center-screen--contained=contained>
            {children()}
        </div>
    }
}

/// A centred card for a sign-in form or a gate state.
///
/// - `brand` (optional): the product mark, at the top.
/// - `title`: an `<h1>`. `sub` (optional): a line under it.
/// - `children`: the form, a `Loading`, an `Alert`, a button.
/// - `footer` (optional): a line at the bottom (a link to help, the
///   version).
#[component]
pub fn AuthCard(
    #[prop(into)] title: String,
    #[prop(optional, into)] sub: String,
    #[prop(optional)] brand: Option<Children>,
    #[prop(optional)] footer: Option<Children>,
    children: Children,
) -> impl IntoView {
    let has_sub = !sub.is_empty();
    view! {
        <div class="cl-auth-card">
            {brand.map(|b| view! { <div class="cl-auth-card__brand">{b()}</div> })}
            <h1 class="cl-auth-card__title">{title}</h1>
            {has_sub.then(|| view! { <p class="cl-auth-card__sub">{sub}</p> })}
            <div class="cl-auth-card__body">{children()}</div>
            {footer.map(|f| view! { <div class="cl-auth-card__footer">{f()}</div> })}
        </div>
    }
}

// ============================================================================
// Tests (pure logic)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trap_wraps_forward_from_last_to_first() {
        assert_eq!(trap_step(3, Some(2), false), TrapStep::Focus(0));
    }

    #[test]
    fn trap_wraps_backward_from_first_to_last() {
        assert_eq!(trap_step(3, Some(0), true), TrapStep::Focus(2));
    }

    #[test]
    fn trap_lets_the_browser_move_in_the_middle() {
        assert_eq!(trap_step(3, Some(1), false), TrapStep::Native);
        assert_eq!(trap_step(3, Some(1), true), TrapStep::Native);
        assert_eq!(trap_step(3, Some(0), false), TrapStep::Native);
        assert_eq!(trap_step(3, Some(2), true), TrapStep::Native);
    }

    #[test]
    fn trap_pulls_focus_in_from_outside() {
        assert_eq!(trap_step(4, None, false), TrapStep::Focus(0));
        assert_eq!(trap_step(4, None, true), TrapStep::Focus(3));
    }

    #[test]
    fn trap_with_no_focusable_keeps_focus() {
        assert_eq!(trap_step(0, None, false), TrapStep::Stay);
        assert_eq!(trap_step(0, None, true), TrapStep::Stay);
    }

    #[test]
    fn trap_with_one_focusable_stays_on_it() {
        assert_eq!(trap_step(1, Some(0), false), TrapStep::Focus(0));
        assert_eq!(trap_step(1, Some(0), true), TrapStep::Focus(0));
    }

    #[test]
    fn tab_keys_move_and_wrap() {
        let d = [false, false, false];
        assert_eq!(tab_key_target("ArrowRight", 0, &d), Some(1));
        assert_eq!(tab_key_target("ArrowRight", 2, &d), Some(0));
        assert_eq!(tab_key_target("ArrowLeft", 0, &d), Some(2));
        assert_eq!(tab_key_target("ArrowLeft", 2, &d), Some(1));
        assert_eq!(tab_key_target("Home", 2, &d), Some(0));
        assert_eq!(tab_key_target("End", 0, &d), Some(2));
    }

    #[test]
    fn tab_keys_skip_disabled_tabs() {
        let d = [false, true, false, true];
        assert_eq!(tab_key_target("ArrowRight", 0, &d), Some(2));
        assert_eq!(tab_key_target("ArrowRight", 2, &d), Some(0));
        assert_eq!(tab_key_target("ArrowLeft", 0, &d), Some(2));
        assert_eq!(tab_key_target("End", 0, &d), Some(2));
        assert_eq!(tab_key_target("Home", 2, &[true, false]), Some(1));
    }

    #[test]
    fn tab_keys_ignore_other_keys_and_empty_lists() {
        assert_eq!(tab_key_target("a", 0, &[false, false]), None);
        assert_eq!(tab_key_target("ArrowDown", 0, &[false, false]), None);
        assert_eq!(tab_key_target("ArrowRight", 0, &[]), None);
        assert_eq!(tab_key_target("ArrowRight", 0, &[true, true]), None);
    }

    #[test]
    fn tab_ids_are_plain_identifiers() {
        let id = tab_dom_id("cl-tabs-3", "run history/1", "panel");
        assert_eq!(id, "cl-tabs-3-panel-run-history-1");
    }

    #[test]
    fn confirm_check() {
        assert!(confirm_matches("", ""));
        assert!(confirm_matches("anything", "  "));
        assert!(confirm_matches("nightly-ingest", "nightly-ingest"));
        assert!(confirm_matches(" nightly-ingest ", "nightly-ingest"));
        assert!(!confirm_matches("nightly", "nightly-ingest"));
        assert!(!confirm_matches("Nightly-Ingest", "nightly-ingest"));
    }

    #[test]
    fn toast_queue_push_and_dismiss() {
        let mut q = ToastQueue::new(5);
        let a = q.push(ToastKind::Info, "a", None);
        let b = q.push(ToastKind::Error, "b", Some(0));
        assert_ne!(a, b);
        assert_eq!(q.items().len(), 2);
        assert!(q.dismiss(a));
        assert!(!q.dismiss(a));
        assert_eq!(q.items()[0].text, "b");
        assert_eq!(q.items()[0].timeout_ms, Some(0));
        q.clear();
        assert!(q.items().is_empty());
    }

    #[test]
    fn toast_queue_drops_the_oldest_when_full() {
        let mut q = ToastQueue::new(3);
        for n in 0..5 {
            q.push(ToastKind::Success, format!("t{n}"), None);
        }
        let texts: Vec<_> = q.items().iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, ["t2", "t3", "t4"]);
    }

    #[test]
    fn toast_ids_are_not_reused() {
        let mut q = ToastQueue::new(1);
        let a = q.push(ToastKind::Info, "a", None);
        let b = q.push(ToastKind::Info, "b", None);
        q.dismiss(b);
        let c = q.push(ToastKind::Info, "c", None);
        assert!(a < b && b < c);
    }

    #[test]
    fn toast_kinds_have_roles_and_hues() {
        assert_eq!(ToastKind::Error.role(), "alert");
        assert_eq!(ToastKind::Info.role(), "status");
        assert_eq!(ToastKind::Success.color(), token::OK);
        assert_eq!(ToastKind::Warning.color(), token::GOLD);
    }

    #[test]
    fn pause_keeps_the_time_left() {
        assert_eq!(remaining_after_pause(5000.0, 1000.0, 3000.0), 3000.0);
        assert_eq!(remaining_after_pause(5000.0, 1000.0, 9000.0), 0.0);
        // A clock that goes back does not add time.
        assert_eq!(remaining_after_pause(5000.0, 3000.0, 1000.0), 5000.0);
    }
}
