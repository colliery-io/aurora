//! Aurora components — core Leptos primitives.
//!
//! Static styling comes from the shared CSS classes; only the component
//! logic/markup differs. Leptos uses fine-grained signals + the `view!` macro,
//! so reactivity is expressed with closures and `Callback`s rather than
//! re-rendering whole components.

use leptos::context::Provider;
use leptos::prelude::*;

use crate::tokens::{classify, fg_for, fill_for, tint, ApiError};

// The frame (AppShell, SideNav, PageHeader, Modal, Drawer, ConfirmDialog,
// toasts, Tabs, Card) lives in `frame.rs`; it is part of this module's API.
pub use crate::frame::*;
// The data components (StatTile, LogView, Pagination, ...) live in `data.rs`,
// the icons in `icons.rs`; both are part of this module's API.
pub use crate::data::*;
pub use crate::icons::*;

/// A unique `id` for one field instance, so a `<label for=…>` can point at its
/// control.
///
/// Leptos has no `useId`, so the counter lives here. A process-wide
/// `AtomicUsize` is enough: ids only have to be unique within a document, and a
/// CSR bundle is one document. Server-side rendering would want the id to be
/// stable across the render/hydrate pair — it is, because both passes call this
/// in the same order.
///
/// Why this exists at all: every labelled field in this library rendered a
/// `<label>` with no `for` and a control with no `id`, so the accessible name of
/// every text input, select and textarea built with Aurora was **empty**. A
/// screen reader announced an unlabelled field; clicking the label did nothing.
/// Found when a Kairos end-to-end test reached for Playwright's `getByLabel` and
/// timed out while everything around it resolved (KAIROS-T-0198).
pub(crate) fn field_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("cl-field-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

// ----------------------------------------------------------------------------
// Layout: Group / Stack
// ----------------------------------------------------------------------------

/// A horizontal flex row.
///
/// - `justify`: `"between"` or `"end"` (default: the start).
/// - `align`: the cross axis. `"center"` (default), `"start"`, `"end"` (the
///   bottom edge: a labelled field and a button line up on the control),
///   `"baseline"` or `"stretch"`. `top=true` is the same as `align="start"`.
/// - `wrap`, `gap` (`"xs"`, `"sm"`; default `md`).
#[component]
pub fn Group(
    #[prop(optional, into)] justify: String,
    #[prop(optional)] top: bool,
    #[prop(optional, into)] align: String,
    #[prop(optional)] wrap: bool,
    #[prop(optional, into)] gap: String,
    children: Children,
) -> impl IntoView {
    let mut class = String::from("cl-group");
    match justify.as_str() {
        "between" => class.push_str(" cl-group--between"),
        "end" => class.push_str(" cl-group--end"),
        _ => {}
    }
    match (top, align.as_str()) {
        (true, _) | (_, "start" | "top") => class.push_str(" cl-group--top"),
        (_, "end" | "bottom") => class.push_str(" cl-group--bottom"),
        (_, "baseline") => class.push_str(" cl-group--baseline"),
        (_, "stretch") => class.push_str(" cl-group--stretch"),
        _ => {}
    }
    if wrap {
        class.push_str(" cl-group--wrap");
    }
    match gap.as_str() {
        "xs" => class.push_str(" cl-group--gap-xs"),
        "sm" => class.push_str(" cl-group--gap-sm"),
        _ => {}
    }
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn Stack(
    #[prop(optional)] center: bool,
    #[prop(optional, into)] gap: String,
    children: Children,
) -> impl IntoView {
    let mut class = String::from("cl-stack");
    if center {
        class.push_str(" cl-stack--center");
    }
    match gap.as_str() {
        "xs" => class.push_str(" cl-stack--gap-xs"),
        "sm" => class.push_str(" cl-stack--gap-sm"),
        _ => {}
    }
    view! { <div class=class>{children()}</div> }
}

// ----------------------------------------------------------------------------
// Text / MONO
// ----------------------------------------------------------------------------

#[component]
pub fn Text(
    #[prop(optional, into)] size: String,
    #[prop(optional)] dimmed: bool,
    #[prop(optional)] bright: bool,
    #[prop(optional)] bold: bool,
    #[prop(optional)] mono: bool,
    children: Children,
) -> impl IntoView {
    let mut class = String::from("cl-text");
    match size.as_str() {
        "xs" => class.push_str(" cl-text--xs"),
        "sm" => class.push_str(" cl-text--sm"),
        "lg" => class.push_str(" cl-text--lg"),
        _ => {}
    }
    if dimmed {
        class.push_str(" cl-text--dimmed");
    }
    if bright {
        class.push_str(" cl-text--bright");
    }
    if bold {
        class.push_str(" cl-text--bold");
    }
    if mono {
        class.push_str(" cl-mono");
    }
    view! { <p class=class>{children()}</p> }
}

// ----------------------------------------------------------------------------
// Button
// ----------------------------------------------------------------------------

/// `Some(s)` when `s` is not empty: an attribute that is left out when the
/// prop is not given.
fn attr(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}

/// A button.
///
/// - `variant`: `"filled"` (default), `"light"`, `"default"` or `"subtle"`.
///   `"primary"` is the same as `"filled"`.
/// - `size`: `"xs"`, `"sm"` (default) or `"md"`. `bad`: the danger colour.
/// - `disabled`: a `bool`, a signal or a closure (`disabled=move || busy.get()`).
/// - `loading`: while true, the button shows a spinner, is disabled and has
///   `aria-busy="true"`. With `loading_label`, that text replaces the
///   children while it loads ("Saving…").
/// - `button_type`: the `type` attribute (`"submit"` in a form). Not set by
///   default, so a button in a `<form>` submits it, as before.
/// - `href`: draws a link (`<a>`) that looks like the button.
/// - `title`, `aria_label`: the tooltip and the accessible name (give
///   `aria_label` when the children are only an icon).
/// - `stop_propagation`: the click does not go on to a parent (a button in
///   a clickable table row or card).
#[component]
pub fn Button(
    #[prop(default = "filled".to_string(), into)] variant: String,
    #[prop(default = "sm".to_string(), into)] size: String,
    #[prop(optional)] bad: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] loading: Signal<bool>,
    #[prop(optional, into)] loading_label: String,
    #[prop(optional, into)] button_type: String,
    #[prop(optional, into)] href: String,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional)] stop_propagation: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let variant = match variant.as_str() {
        "primary" | "" => "filled".to_string(),
        _ => variant,
    };
    let mut class = format!("cl-btn cl-btn--{variant}");
    if size != "sm" {
        class.push_str(&format!(" cl-btn--{size}"));
    }
    if bad {
        class.push_str(" cl-btn--bad");
    }
    let off = move || disabled.get() || loading.get();
    let has_loading_label = !loading_label.is_empty();
    let content = view! {
        {move || loading.get().then(|| view! { <span class="cl-btn__spinner" aria-hidden="true"></span> })}
        <span class="cl-btn__label" hidden=move || has_loading_label && loading.get()>{children()}</span>
        {has_loading_label.then(|| view! {
            <span class="cl-btn__label" hidden=move || !loading.get()>{loading_label}</span>
        })}
    };
    let click = move |ev: web_sys::MouseEvent| {
        if stop_propagation {
            ev.stop_propagation();
        }
        if off() {
            ev.prevent_default();
            return;
        }
        if let Some(cb) = on_click {
            cb.run(());
        }
    };
    if !href.is_empty() {
        return view! {
            <a
                class=class
                href=href
                title=attr(title)
                aria-label=attr(aria_label)
                aria-disabled=move || off().then_some("true")
                aria-busy=move || loading.get().then_some("true")
                tabindex=move || off().then_some("-1")
                on:click=click
            >
                {content}
            </a>
        }
        .into_any();
    }
    view! {
        <button
            class=class
            type=attr(button_type)
            title=attr(title)
            aria-label=attr(aria_label)
            disabled=off
            aria-busy=move || if loading.get() { "true" } else { "false" }
            on:click=click
        >
            {content}
        </button>
    }
    .into_any()
}

// ----------------------------------------------------------------------------
// TextInput / Select
// ----------------------------------------------------------------------------

/// The id of a control: the `id` prop when a product gives one (so a label
/// outside the component can point at it), else a new unique id.
fn control_id(id: String) -> String {
    if id.is_empty() {
        field_id()
    } else {
        id
    }
}

/// The message under a field, while `error` is not empty. `error` is a
/// signal, so the message can come and go (AURORA-T-0007 item 12).
fn field_error(error: Signal<String>, err_id: String) -> impl IntoView {
    move || {
        let e = error.get();
        (!e.is_empty()).then(|| {
            view! { <span class="cl-field__error" id=err_id.clone()>{e}</span> }
        })
    }
}

/// A one-line text field, bound to `value`.
///
/// - `label`, `placeholder`, `error` (a red border and a message under it).
///   `error` takes a `String`, a `&str`, a signal or a closure, so the
///   message can change while the field is shown.
/// - `id`: the `id` of the `<input>`, for a `<label for=…>` outside the
///   component. Default: a unique id.
/// - `aria_label`: the accessible name when there is no visible `label`.
/// - `input_type`: the `type` attribute, `"text"` by default (`"email"`,
///   `"url"`, `"search"`, `"date"`, `"password"`...).
/// - `disabled`: a `bool`, a signal or a closure.
/// - `on_input`: runs with the new text on each key press (after `value`
///   changes). `on_change`: runs when the person leaves the field or presses
///   Enter after a change.
/// - `name`, `autocomplete`, `required`, `spellcheck`: the HTML attributes
///   (for forms and password managers).
/// - `mono`: the text in the mono font (ids, codes).
#[component]
pub fn TextInput(
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] input_type: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] autocomplete: String,
    #[prop(optional)] required: bool,
    #[prop(optional)] spellcheck: Option<bool>,
    #[prop(optional)] mono: bool,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
) -> impl IntoView {
    let mut input_class = String::from("cl-input");
    if mono {
        input_class.push_str(" cl-mono");
    }
    let input_type = if input_type.is_empty() {
        "text".to_string()
    } else {
        input_type
    };
    let has_label = !label.is_empty();
    let has_error = move || error.with(|e| !e.is_empty());
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let described = err_id.clone();
    view! {
        <div class="cl-field">
            {has_label.then(|| view! {
                <label class="cl-field__label" for=id.clone()>{label}</label>
            })}
            <input
                class=input_class
                class:cl-input--error=has_error
                id=id
                type=input_type
                placeholder=attr(placeholder)
                name=attr(name)
                autocomplete=attr(autocomplete)
                aria-label=attr(aria_label)
                required=required
                spellcheck=spellcheck.map(|s| if s { "true" } else { "false" })
                aria-invalid=move || has_error().then_some("true")
                aria-describedby=move || has_error().then(|| described.clone())
                disabled=move || disabled.get()
                prop:value=move || value.get()
                on:input=move |e| {
                    let v = event_target_value(&e);
                    value.set(v.clone());
                    if let Some(cb) = on_input { cb.run(v); }
                }
                on:change=move |e| {
                    if let Some(cb) = on_change { cb.run(event_target_value(&e)); }
                }
            />
            {field_error(error, err_id)}
        </div>
    }
}

/// The value that a [`Select`] shows: `value` when an option has it (or when
/// there is a placeholder, whose value is `""`), else the first option, as a
/// native `<select>` does (AURORA-T-0007 item 8).
pub fn select_shown_value(value: &str, options: &[String], has_placeholder: bool) -> String {
    if has_placeholder || options.iter().any(|o| o == value) {
        return value.to_string();
    }
    options
        .first()
        .cloned()
        .unwrap_or_else(|| value.to_string())
}

/// A native select, bound to `value`.
///
/// - `options`: the choices, where the value and the text are the same.
/// - `option_pairs`: `(value, label)` choices, where the text differs from
///   the value (`("42", "COLLIERY-T-42 · Fix the pager")`). Shown after
///   `options`.
/// - `placeholder`: a first choice with the value `""` (such as "Choose a
///   board" or "(none)").
/// - `aria_label`: the accessible name when there is no visible `label`.
///   (`attr:aria-label` goes on the wrapper `<div>`, not on the `<select>`.)
/// - `id`: the `id` of the `<select>`, for a `<label for=…>` outside the
///   component. Default: a unique id.
/// - `disabled`, `on_change` (runs with the new value), `name`, `required`,
///   `error` (a `String`, a `&str`, a signal or a closure).
///
/// When `value` matches no option and there is no placeholder, the select
/// shows the first option, as a native select does. `value` itself is not
/// changed.
#[component]
pub fn Select(
    #[prop(optional, into)] label: String,
    #[prop(optional)] options: Vec<String>,
    #[prop(optional)] option_pairs: Vec<(String, String)>,
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional, into)] name: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
) -> impl IntoView {
    let has_label = !label.is_empty();
    let has_error = move || error.with(|e| !e.is_empty());
    let pairs = options
        .into_iter()
        .map(|o| (o.clone(), o))
        .chain(option_pairs)
        .collect::<Vec<_>>();
    let has_placeholder = !placeholder.is_empty();
    let values = StoredValue::new(pairs.iter().map(|(v, _)| v.clone()).collect::<Vec<_>>());
    let shown = Memo::new(move |_| {
        values.with_value(|vs| select_shown_value(&value.get(), vs, has_placeholder))
    });
    let placeholder_opt = has_placeholder
        .then(|| view! { <option value="" selected=move || shown.get().is_empty()>{placeholder}</option> });
    let opts = pairs
        .into_iter()
        .map(|(v, l)| {
            let sel = v.clone();
            // `selected` as well as `prop:value` on the select: the value is
            // right on the first render, before the options exist.
            view! { <option value=v selected=move || shown.with(|s| *s == sel)>{l}</option> }
        })
        .collect_view();
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let described = err_id.clone();
    view! {
        <div class="cl-field">
            {has_label.then(|| view! {
                <label class="cl-field__label" for=id.clone()>{label}</label>
            })}
            <select
                class="cl-input cl-select"
                class:cl-input--error=has_error
                id=id
                name=attr(name)
                aria-label=attr(aria_label)
                required=required
                aria-invalid=move || has_error().then_some("true")
                aria-describedby=move || has_error().then(|| described.clone())
                disabled=move || disabled.get()
                prop:value=move || shown.get()
                on:change=move |e| {
                    let v = event_target_value(&e);
                    value.set(v.clone());
                    if let Some(cb) = on_change { cb.run(v); }
                }
            >
                {placeholder_opt}
                {opts}
            </select>
            {field_error(error, err_id)}
        </div>
    }
}

// ----------------------------------------------------------------------------
// Tooltip
// ----------------------------------------------------------------------------

/// A short text that shows over its trigger on hover and on keyboard focus.
///
/// - `label`: the text. It is also the accessible description of the
///   trigger (`aria-describedby` on the first control inside).
/// - `position`: `"top"` (default), `"bottom"`, `"left"` or `"right"`.
/// - `focusable`: the wrapper takes keyboard focus. Use it when the trigger
///   is not a control (a pill, an icon, an inline SVG).
///
/// It shows after a short delay, and at once for keyboard focus. Escape
/// hides it. It is pure CSS, so a parent with `overflow: hidden` can cut it:
/// use `position` to keep it inside. For a plain hint on truncated text, a
/// `title` attribute is enough. Inside an SVG chart, use an SVG `<title>`.
#[component]
pub fn Tooltip(
    #[prop(into)] label: String,
    #[prop(optional, into)] position: String,
    #[prop(optional)] focusable: bool,
    children: Children,
) -> impl IntoView {
    let pos = match position.as_str() {
        "bottom" | "left" | "right" => position,
        _ => "top".to_string(),
    };
    let id = field_id().replace("cl-field-", "cl-tip-");
    let root = NodeRef::<leptos::html::Span>::new();
    let dismissed = RwSignal::new(false);
    // Describe the first control inside by the tooltip text.
    let tip_id = id.clone();
    Effect::new(move |_| {
        if focusable {
            return;
        }
        if let Some(el) = root.get() {
            if let Ok(Some(ctl)) =
                el.query_selector("a[href], button, input, select, textarea, [tabindex]")
            {
                let _ = ctl.set_attribute("aria-describedby", &tip_id);
            }
        }
    });
    view! {
        <span
            node_ref=root
            class=format!("cl-tooltip cl-tooltip--{pos}")
            class:cl-tooltip--dismissed=move || dismissed.get()
            tabindex=focusable.then_some("0")
            aria-describedby=focusable.then(|| id.clone())
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" { dismissed.set(true); }
            }
            on:mouseleave=move |_| dismissed.set(false)
            on:focusout=move |_| dismissed.set(false)
        >
            {children()}
            <span class="cl-tooltip__label" role="tooltip" id=id.clone()>{label}</span>
        </span>
    }
}

// ----------------------------------------------------------------------------
// Aurora: Pill / StatusBadge / Dot / Panel / Chip
// ----------------------------------------------------------------------------

#[component]
pub fn Pill(#[prop(into)] color: String, children: Children) -> impl IntoView {
    let style = format!("background:{};color:{};", fill_for(&color), fg_for(&color));
    view! { <span class="cl-pill" style=style>{children()}</span> }
}

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let color = crate::tokens::status_color(&status);
    let style = format!("background:{};color:{};", fill_for(color), fg_for(color));
    view! { <span class="cl-status-badge" style=style>{status}</span> }
}

#[component]
pub fn Dot(
    #[prop(into)] color: String,
    #[prop(default = 8)] size: i32,
    #[prop(optional)] glow: bool,
) -> impl IntoView {
    // `color` too: the hyper glow draws in currentColor.
    let mut style = format!("width:{size}px;height:{size}px;background:{color};color:{color};");
    if glow {
        style.push_str(&format!("box-shadow:0 0 0 3px {};", tint(&color, 13)));
    }
    view! { <span class="cl-dot" style=style></span> }
}

#[component]
pub fn Panel(
    #[prop(into)] title: String,
    #[prop(optional, into)] caption: String,
    children: Children,
) -> impl IntoView {
    let has_caption = !caption.is_empty();
    view! {
        <div class="cl-panel">
            <div class="cl-panel__header">
                <span class="cl-panel__title">{title}</span>
                {has_caption.then(|| view! { <span class="cl-panel__caption">{caption}</span> })}
            </div>
            {children()}
        </div>
    }
}

/// A filter chip: a toggle button.
///
/// - `label`, `count` (shown when 0 or more), `active`, `on_click`.
///
/// It is a `type="button"` (it does not submit a form) with `aria-pressed`
/// from `active`, so a screen reader says whether the filter is on.
#[component]
pub fn Chip(
    #[prop(into)] label: String,
    #[prop(default = -1)] count: i32,
    #[prop(into)] active: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="cl-chip"
            class:cl-chip--active=move || active.get()
            aria-pressed=move || if active.get() { "true" } else { "false" }
            on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
        >
            {label}
            {(count >= 0).then(|| view! { <span class="cl-chip__count">{count}</span> })}
        </button>
    }
}

// ----------------------------------------------------------------------------
// States: Loading / Empty / ErrorState
// ----------------------------------------------------------------------------

#[component]
pub fn Loading(#[prop(default = "Loading…".to_string(), into)] label: String) -> impl IntoView {
    view! {
        <div class="cl-center">
            <Stack center=true gap="xs">
                <div class="cl-loader"></div>
                <Text dimmed=true size="sm">{label}</Text>
            </Stack>
        </div>
    }
}

/// An empty state: a message, and an optional next step.
///
/// - `message`: what is empty ("No agents yet.").
/// - `hint` (optional): the next step, in a smaller line under it ("Install
///   an agent to see it here.").
/// - `href` + `link` (optional): a link after the hint ("Read how", the
///   default text when only `href` is given).
/// - children (optional): any next-step content (a button, a link) under the
///   message and the hint.
#[component]
pub fn Empty(
    #[prop(into)] message: String,
    #[prop(optional, into)] hint: String,
    #[prop(optional, into)] href: String,
    #[prop(optional, into)] link: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let anchor = (!href.is_empty()).then(|| {
        let text = if link.is_empty() {
            "Read how".to_string()
        } else {
            link
        };
        view! { <Anchor href=href>{text}</Anchor> }
    });
    let has_next = !hint.is_empty() || anchor.is_some();
    let next = has_next.then(|| {
        let gap = (!hint.is_empty() && anchor.is_some()).then_some(" ");
        view! { <p class="cl-empty__next">{hint}{gap}{anchor}</p> }
    });
    view! {
        <div class="cl-center cl-empty">
            <Text dimmed=true>{message}</Text>
            {next}
            {children.map(|c| view! { <div class="cl-empty__actions">{c()}</div> })}
        </div>
    }
}

#[component]
pub fn ErrorState(
    error: ApiError,
    #[prop(optional)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    let c = classify(&error);
    let style = format!("--alert-color:var({});", c.color_var);
    let code = c.code.clone();
    view! {
        <div class="cl-alert" style=style role="alert">
            <div class="cl-alert__title">{c.title}</div>
            <Stack gap="xs">
                <Text size="sm">{c.message}</Text>
                {code.map(|code| view! { <Text size="xs" dimmed=true>{format!("code: {code}")}</Text> })}
                {c.retryable.then(|| view! {
                    // tinted to the alert color (inherits --alert-color) — no blue/red clash
                    <button
                        class="cl-btn cl-btn--xs cl-btn--alert"
                        style="width:fit-content;"
                        on:click=move |_| { if let Some(cb) = on_retry { cb.run(()); } }
                    >
                        "Retry"
                    </button>
                })}
            </Stack>
        </div>
    }
}

// ----------------------------------------------------------------------------
// Additional Mantine primitives (completing the inventory)
// ----------------------------------------------------------------------------

/// Plain block / style carrier (Mantine `Box`).
#[component]
pub fn Box(children: Children) -> impl IntoView {
    view! { <div class="cl-box">{children()}</div> }
}

/// Inline monospace `<code>` chip (Mantine `Code`).
#[component]
pub fn Code(children: Children) -> impl IntoView {
    view! { <code class="cl-code">{children()}</code> }
}

/// Accent text link (Mantine `Anchor`).
#[component]
pub fn Anchor(#[prop(optional, into)] href: String, children: Children) -> impl IntoView {
    let href = if href.is_empty() {
        "#".to_string()
    } else {
        href
    };
    view! { <a class="cl-anchor" href=href>{children()}</a> }
}

/// Hairline rule (Mantine `Divider`).
#[component]
pub fn Divider() -> impl IntoView {
    view! { <hr class="cl-divider" /> }
}

/// Standalone spinner (Mantine `Loader`).
#[component]
pub fn Loader() -> impl IntoView {
    view! { <div class="cl-loader"></div> }
}

/// Square icon-only button (Mantine `ActionIcon`).
#[component]
pub fn ActionIcon(
    #[prop(optional, into)] title: String,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="cl-action-icon"
            title=title
            on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
        >
            {children()}
        </button>
    }
}

/// Tinted callout (Mantine `Alert`). `color` is a hex/token value used for the
/// accent (defaults to the bad/red token). Powers `ErrorState` too.
#[component]
pub fn Alert(
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] color: String,
    children: Children,
) -> impl IntoView {
    let style = if color.is_empty() {
        String::new()
    } else {
        format!("--alert-color:{color};")
    };
    let has_title = !title.is_empty();
    view! {
        <div class="cl-alert" style=style role="alert">
            {has_title.then(|| view! { <div class="cl-alert__title">{title}</div> })}
            {children()}
        </div>
    }
}

/// Toggle (Mantine `Switch`), controlled by a bool signal.
///
/// A real `<button role="switch">`, not a clickable `<span>` — which is what this
/// was (KAIROS-T-0198). The span version had a worse problem than the missing
/// label association that ticket was filed about: it was not a control at all. No
/// role, no state, not focusable, not keyboard-operable. A keyboard user could not
/// toggle it and a screen reader saw two pieces of decorative text.
///
/// A `<button>` gets focus, Enter and Space for free; `role="switch"` plus
/// `aria-checked` gives it the on/off state; and the label lives INSIDE the
/// button, which makes it the accessible name with no `for`/`id` needed. The
/// label is also then part of the click target, which it always looked like it
/// was.
///
/// - `disabled`: a `bool`, a signal or a closure.
/// - `on_change`: runs with the new state after a click.
#[component]
pub fn Switch(
    checked: RwSignal<bool>,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
) -> impl IntoView {
    let has_label = !label.is_empty();
    view! {
        <button
            type="button"
            role="switch"
            class="cl-switch"
            class:cl-switch--on=move || checked.get()
            aria-checked=move || if checked.get() { "true" } else { "false" }
            disabled=move || disabled.get()
            on:click=move |_| {
                checked.update(|v| *v = !*v);
                if let Some(cb) = on_change { cb.run(checked.get_untracked()); }
            }
        >
            <span class="cl-switch__track"><span class="cl-switch__thumb"></span></span>
            {has_label.then(|| view! { <span class="cl-text cl-text--sm">{label}</span> })}
        </button>
    }
}

/// Segmented selector (Mantine `SegmentedControl`), bound to a string signal.
#[component]
pub fn SegmentedControl(options: Vec<String>, value: RwSignal<String>) -> impl IntoView {
    let items = options
        .into_iter()
        .map(|opt| {
            let o = opt.clone();
            let o_aria = opt.clone();
            let o2 = opt.clone();
            view! {
                <button
                    type="button"
                    class="cl-segmented__item"
                    class:cl-segmented__item--active=move || value.get() == o
                    // KAIROS-T-0198: these were already real buttons, so focus and
                    // keyboard worked — but WHICH one is selected was conveyed by
                    // colour alone. aria-pressed says it out loud.
                    aria-pressed=move || if value.get() == o_aria { "true" } else { "false" }
                    on:click=move |_| value.set(o2.clone())
                >
                    {opt}
                </button>
            }
        })
        .collect_view();
    view! { <div class="cl-segmented">{items}</div> }
}

/// Multi-line text field (Mantine `Textarea`), bound to `value`.
///
/// Same props as [`TextInput`] where they apply: `disabled`, `on_input`,
/// `on_change`, `name`, `required`, `error` (a string, a signal or a
/// closure), `mono`, `id`, `aria_label`. `rows` (default 4).
#[component]
pub fn Textarea(
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(default = 4)] rows: i32,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional, into)] name: String,
    #[prop(optional)] required: bool,
    #[prop(optional)] mono: bool,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
) -> impl IntoView {
    let has_label = !label.is_empty();
    let has_error = move || error.with(|e| !e.is_empty());
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let described = err_id.clone();
    let mut class = String::from("cl-input");
    if mono {
        class.push_str(" cl-mono");
    }
    view! {
        <div class="cl-field">
            {has_label.then(|| view! {
                <label class="cl-field__label" for=id.clone()>{label}</label>
            })}
            <textarea
                class=class
                    class:cl-input--error=has_error
                id=id
                rows=rows
                placeholder=attr(placeholder)
                name=attr(name)
                aria-label=attr(aria_label)
                required=required
                aria-invalid=move || has_error().then_some("true")
                aria-describedby=move || has_error().then(|| described.clone())
                disabled=move || disabled.get()
                prop:value=move || value.get()
                on:input=move |e| {
                    let v = event_target_value(&e);
                    value.set(v.clone());
                    if let Some(cb) = on_input { cb.run(v); }
                }
                on:change=move |e| {
                    if let Some(cb) = on_change { cb.run(event_target_value(&e)); }
                }
            ></textarea>
            {field_error(error, err_id)}
        </div>
    }
}

/// Numeric field with steppers (Mantine `NumberInput`), bound to an f64 signal.
///
/// - `step` (default 1), `min` and `max` (the steppers stop there).
/// - `disabled`, `on_change` (runs with the new number), `name`, `required`,
///   `error` (a string, a signal or a closure), `id` (of the `<input>`).
#[component]
pub fn NumberInput(
    #[prop(optional, into)] label: String,
    value: RwSignal<f64>,
    #[prop(default = 1.0)] step: f64,
    #[prop(optional)] min: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<f64>>,
    #[prop(optional, into)] name: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let has_label = !label.is_empty();
    let has_error = move || error.with(|e| !e.is_empty());
    let fmt = move || {
        let v = value.get();
        if v.fract() == 0.0 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        }
    };
    let clamp = move |v: f64| {
        let v = min.map_or(v, |m| v.max(m));
        max.map_or(v, |m| v.min(m))
    };
    let set = move |v: f64| {
        let v = clamp(v);
        value.set(v);
        if let Some(cb) = on_change {
            cb.run(v);
        }
    };
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let described = err_id.clone();
    let class = "cl-input";
    view! {
        <div class="cl-field">
            {has_label.then(|| view! {
                <label class="cl-field__label" for=id.clone()>{label}</label>
            })}
            <div class="cl-number">
                <input
                    class=class
                    class:cl-input--error=has_error
                    id=id
                    type="number"
                    step=step
                    min=min
                    max=max
                    name=attr(name)
                    required=required
                    aria-invalid=move || has_error().then_some("true")
                    aria-describedby=move || has_error().then(|| described.clone())
                    disabled=move || disabled.get()
                    prop:value=fmt
                    on:input=move |e| {
                        if let Ok(n) = event_target_value(&e).parse::<f64>() { set(n); }
                    }
                />
                <div class="cl-number__steps">
                    // KAIROS-T-0198: type="button" so they do not submit a
                    // surrounding form, and named, because "▲" is not a name.
                    <button
                        type="button"
                        class="cl-number__step"
                        aria-label="Increase"
                        disabled=move || disabled.get()
                        on:click=move |_| set(value.get_untracked() + step)
                    >"▲"</button>
                    <button
                        type="button"
                        class="cl-number__step"
                        aria-label="Decrease"
                        disabled=move || disabled.get()
                        on:click=move |_| set(value.get_untracked() - step)
                    >"▼"</button>
                </div>
            </div>
            {field_error(error, err_id)}
        </div>
    }
}

/// Password field with a reveal toggle (Mantine `PasswordInput`).
///
/// - `autocomplete`: `"current-password"` on a sign-in form,
///   `"new-password"` on a form that sets one, so a password manager helps.
/// - `disabled`, `on_input`, `on_change`, `name`, `required`, `error` (a
///   string, a signal or a closure), `id` (of the `<input>`).
#[component]
pub fn PasswordInput(
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] autocomplete: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let reveal = RwSignal::new(false);
    let has_label = !label.is_empty();
    let has_error = move || error.with(|e| !e.is_empty());
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let described = err_id.clone();
    let class = "cl-input";
    view! {
        <div class="cl-field">
            {has_label.then(|| view! {
                <label class="cl-field__label" for=id.clone()>{label}</label>
            })}
            <div class="cl-input-wrap">
                <input
                    class=class
                    class:cl-input--error=has_error
                    id=id
                    type=move || if reveal.get() { "text" } else { "password" }
                    placeholder=attr(placeholder)
                    name=attr(name)
                    autocomplete=attr(autocomplete)
                    required=required
                    aria-invalid=move || has_error().then_some("true")
                    aria-describedby=move || has_error().then(|| described.clone())
                    disabled=move || disabled.get()
                    prop:value=move || value.get()
                    on:input=move |e| {
                        let v = event_target_value(&e);
                        value.set(v.clone());
                        if let Some(cb) = on_input { cb.run(v); }
                    }
                    on:change=move |e| {
                        if let Some(cb) = on_change { cb.run(event_target_value(&e)); }
                    }
                />
                <button
                    class="cl-input-wrap__adorn"
                    type="button"
                    title=move || if reveal.get() { "Hide password" } else { "Show password" }
                    aria-label=move || if reveal.get() { "Hide password" } else { "Show password" }
                    aria-pressed=move || if reveal.get() { "true" } else { "false" }
                    on:click=move |_| reveal.update(|v| *v = !*v)
                >
                    {move || if reveal.get() {
                        view! {
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                <path d="M10.585 10.587a2 2 0 0 0 2.829 2.828" />
                                <path d="M16.681 16.673a8.717 8.717 0 0 1 -4.681 1.327c-4 0 -7.333 -2.333 -10 -7c1.21 -2.12 2.554 -3.685 4.032 -4.692m3.96 -1.051a8.86 8.86 0 0 1 2.008 -.157c4 0 7.333 2.333 10 7c-.474 .83 -.969 1.542 -1.486 2.139" />
                                <path d="M3 3l18 18" />
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                <circle cx="12" cy="12" r="2" />
                                <path d="M22 12c-2.667 4.667 -6 7 -10 7s-7.333 -2.333 -10 -7c2.667 -4.667 6 -7 10 -7s7.333 2.333 10 7" />
                            </svg>
                        }.into_any()
                    }}
                </button>
            </div>
            {field_error(error, err_id)}
        </div>
    }
}

/// Equal-width responsive grid (Mantine `SimpleGrid`).
///
/// - `cols` (default 2): the columns on a wide screen. Below 768px the grid
///   has at most 2 columns, and below 480px one.
/// - `fixed`: keep `cols` at every width (no collapse).
#[component]
pub fn SimpleGrid(
    #[prop(default = 2)] cols: usize,
    #[prop(optional)] fixed: bool,
    children: Children,
) -> impl IntoView {
    let cols = cols.max(1);
    let class = if fixed {
        "cl-simple-grid cl-simple-grid--fixed"
    } else {
        "cl-simple-grid"
    };
    view! { <div class=class style=format!("--cl-cols:{cols};")>{children()}</div> }
}

/// 12-column grid container (Mantine `Grid`). Pair with `GridCol`.
#[component]
pub fn Grid(children: Children) -> impl IntoView {
    view! { <div class="cl-grid">{children()}</div> }
}

/// A column within a `Grid`; `span` is out of 12.
#[component]
pub fn GridCol(#[prop(default = 12)] span: u8, children: Children) -> impl IntoView {
    let pct = (span.min(12) as f64 / 12.0) * 100.0;
    let style = format!("flex:0 0 {pct}%;max-width:{pct}%;");
    view! { <div class="cl-grid__col" style=style>{children()}</div> }
}

/// Bulleted list (Mantine `List`). Use `ListItem` children.
#[component]
pub fn List(children: Children) -> impl IntoView {
    view! { <ul class="cl-list">{children()}</ul> }
}

/// A `List` item.
#[component]
pub fn ListItem(children: Children) -> impl IntoView {
    view! { <li>{children()}</li> }
}

// ----------------------------------------------------------------------------
// Menu
// ----------------------------------------------------------------------------

/// The menu item that a key moves to, or `None` when the key does nothing.
///
/// `current` is the index of the item with focus (`None` when focus is on
/// the trigger). ArrowDown and ArrowUp move and wrap; Home and End go to the
/// first and the last item.
pub fn menu_key_target(key: &str, current: Option<usize>, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let last = count - 1;
    match key {
        "Home" => Some(0),
        "End" => Some(last),
        "ArrowDown" => Some(match current {
            Some(i) if i < last => i + 1,
            _ => 0,
        }),
        "ArrowUp" => Some(match current {
            Some(i) if i > 0 && i <= last => i - 1,
            _ => last,
        }),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct MenuCtx {
    open: RwSignal<bool>,
    trigger: NodeRef<leptos::html::Button>,
}

const MENU_ITEMS: &str = "[role=\"menuitem\"]:not([disabled]):not([aria-disabled=\"true\"])";

fn menu_items(root: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    use leptos::wasm_bindgen::JsCast;
    let Ok(list) = root.query_selector_all(MENU_ITEMS) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
        .collect()
}

/// A dropdown of actions off a trigger button. Put [`MenuItem`]s in it (and
/// [`MenuLabel`] / [`MenuDivider`]).
///
/// - `label`: the text of the default trigger (a default button with a
///   chevron). Or `trigger`: your own trigger content (a tenant name with a
///   dot, an icon); `trigger_class` styles it (default: a plain, full-width
///   row). Give `aria_label` when the trigger is only an icon.
/// - `align`: `"start"` (default) or `"end"` (the right edges line up, for a
///   menu at the right of a bar). `up`: open above the trigger (a menu at
///   the bottom of a sidebar).
/// - `open` (optional): hold the open state in your own signal.
///
/// Keyboard: Enter, Space or ArrowDown on the trigger opens it on the first
/// item, ArrowUp on the last. ArrowUp / ArrowDown / Home / End move. Escape
/// closes it and gives focus back to the trigger; Tab closes it. A click
/// outside closes it. An item runs its action and closes the menu.
#[component]
pub fn Menu(
    #[prop(optional, into)] label: String,
    #[prop(optional)] trigger: Option<Children>,
    #[prop(optional, into)] trigger_class: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional, into)] align: String,
    #[prop(optional)] up: bool,
    #[prop(optional)] open: Option<RwSignal<bool>>,
    children: ChildrenFn,
) -> impl IntoView {
    use leptos::wasm_bindgen::JsCast;

    let open = open.unwrap_or_else(|| RwSignal::new(false));
    let trigger_ref = NodeRef::<leptos::html::Button>::new();
    let root = NodeRef::<leptos::html::Div>::new();
    let list_ref = NodeRef::<leptos::html::Div>::new();
    // Given to the items through a `Provider` (its own owner), not
    // `provide_context` here: a component has no owner of its own, so two
    // menus on one page would share the context of the last one.
    let ctx = MenuCtx {
        open,
        trigger: trigger_ref,
    };
    // Which item takes focus when the list opens: Some(true) first,
    // Some(false) last.
    let focus_on_open = StoredValue::new(None::<bool>);
    let list_id = field_id().replace("cl-field-", "cl-menu-");

    Effect::new(move |_| {
        if open.get() {
            if let Some(first) = focus_on_open.try_update_value(|v| v.take()).flatten() {
                request_animation_frame(move || {
                    if let Some(list) = list_ref.get_untracked() {
                        let items = menu_items(&list);
                        let pick = if first { items.first() } else { items.last() };
                        if let Some(el) = pick {
                            let _ = el.focus();
                        }
                    }
                });
            }
        }
    });

    // A press outside the menu closes it.
    let outside = window_event_listener(leptos::ev::mousedown, move |ev| {
        if !open.get_untracked() {
            return;
        }
        let Some(r) = root.get_untracked() else {
            return;
        };
        let inside = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
            .is_some_and(|n| r.contains(Some(&n)));
        if !inside {
            open.set(false);
        }
    });
    on_cleanup(move || outside.remove());

    let close_to_trigger = move || {
        open.set(false);
        if let Some(t) = trigger_ref.get_untracked() {
            let _ = t.focus();
        }
    };

    let on_trigger_key = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "ArrowDown" | "ArrowUp" => {
            ev.prevent_default();
            focus_on_open.set_value(Some(ev.key() == "ArrowDown"));
            if open.get_untracked() {
                // Already open: the effect does not run again, so move now.
                if let Some(list) = list_ref.get_untracked() {
                    let items = menu_items(&list);
                    let pick = if ev.key() == "ArrowDown" {
                        items.first()
                    } else {
                        items.last()
                    };
                    if let Some(el) = pick {
                        let _ = el.focus();
                    }
                }
            } else {
                open.set(true);
            }
        }
        "Escape" if open.get_untracked() => {
            ev.prevent_default();
            open.set(false);
        }
        _ => {}
    };

    let on_list_key = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        match key.as_str() {
            "Escape" => {
                ev.prevent_default();
                ev.stop_propagation();
                close_to_trigger();
            }
            "Tab" => open.set(false),
            _ => {
                let Some(list) = list_ref.get_untracked() else {
                    return;
                };
                let items = menu_items(&list);
                let active = document().active_element();
                let current = active.and_then(|a| {
                    items
                        .iter()
                        .position(|e| e.unchecked_ref::<web_sys::Node>().is_same_node(Some(&a)))
                });
                if let Some(i) = menu_key_target(&key, current, items.len()) {
                    ev.prevent_default();
                    let _ = items[i].focus();
                }
            }
        }
    };

    let custom = trigger.is_some();
    let trigger_class = if !trigger_class.is_empty() {
        trigger_class
    } else if custom {
        "cl-menu__trigger".to_string()
    } else {
        "cl-btn cl-btn--default".to_string()
    };
    let trigger_content = match trigger {
        Some(t) => t().into_any(),
        None => view! {
            {label}
            <crate::icons::IconChevron size=14 />
        }
        .into_any(),
    };
    let mut drop_class = String::from("cl-menu__dropdown");
    if align == "end" {
        drop_class.push_str(" cl-menu__dropdown--end");
    }
    if up {
        drop_class.push_str(" cl-menu__dropdown--up");
    }
    let children = StoredValue::new(children);
    let list_id2 = list_id.clone();
    view! {
        <div class="cl-menu" class:cl-menu--block=custom node_ref=root>
            <button
                node_ref=trigger_ref
                type="button"
                class=trigger_class
                aria-haspopup="menu"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                aria-controls=list_id
                aria-label=attr(aria_label)
                on:click=move |_| {
                    if !open.get_untracked() {
                        focus_on_open.set_value(Some(true));
                    }
                    open.update(|v| *v = !*v);
                }
                on:keydown=on_trigger_key
            >
                {trigger_content}
            </button>
            {move || open.get().then(|| {
                let drop_class = drop_class.clone();
                view! {
                    <div
                        node_ref=list_ref
                        id=list_id2.clone()
                        class=drop_class
                        role="menu"
                        on:keydown=on_list_key
                    >
                        <Provider value=ctx>{children.with_value(|c| c())}</Provider>
                    </div>
                }
            })}
        </div>
    }
}

/// An item in a [`Menu`]. It runs `on_click` (or opens `href`), closes the
/// menu, and gives focus back to the trigger.
///
/// - `disabled`: a `bool`, a signal or a closure; the item stays in the list
///   but does nothing.
/// - `danger`: the danger colour (delete, remove).
#[component]
pub fn MenuItem(
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] href: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] danger: bool,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<MenuCtx>();
    let activate = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(cb) = on_click {
            cb.run(());
        }
        if let Some(c) = ctx {
            c.open.set(false);
            if let Some(t) = c.trigger.get_untracked() {
                let _ = t.focus();
            }
        }
    };
    if !href.is_empty() {
        return view! {
            <a
                class="cl-menu__item"
                class:cl-menu__item--danger=danger
                role="menuitem"
                tabindex="-1"
                href=href
                aria-disabled=move || disabled.get().then_some("true")
                on:click=move |ev| {
                    if disabled.get_untracked() { ev.prevent_default(); }
                    activate();
                }
            >
                {children()}
            </a>
        }
        .into_any();
    }
    view! {
        <button
            type="button"
            class="cl-menu__item"
            class:cl-menu__item--danger=danger
            role="menuitem"
            tabindex="-1"
            disabled=move || disabled.get()
            on:click=move |_| activate()
        >
            {children()}
        </button>
    }
    .into_any()
}

/// A small heading for a group of items in a [`Menu`].
#[component]
pub fn MenuLabel(children: Children) -> impl IntoView {
    view! { <div class="cl-menu__label" role="presentation">{children()}</div> }
}

/// A line between groups of items in a [`Menu`].
#[component]
pub fn MenuDivider() -> impl IntoView {
    view! { <div class="cl-menu__divider" role="separator"></div> }
}

// ----------------------------------------------------------------------------
// CopyButton
// ----------------------------------------------------------------------------

/// The clipboard, or `None` where the browser does not give it (a page that
/// is not a secure context: plain HTTP that is not localhost).
fn clipboard() -> Option<web_sys::Clipboard> {
    let clip = web_sys::window()?.navigator().clipboard();
    let v: &leptos::wasm_bindgen::JsValue = clip.as_ref();
    (!v.is_undefined() && !v.is_null()).then_some(clip)
}

/// `value` as an absolute URL: a path that starts with `/` gets the origin
/// of the page in front of it.
fn absolute_url(value: &str) -> String {
    if value.starts_with('/') && !value.starts_with("//") {
        if let Some(origin) = web_sys::window().and_then(|w| w.location().origin().ok()) {
            return format!("{origin}{value}");
        }
    }
    value.to_string()
}

/// A button that copies `value` to the clipboard and says so.
///
/// - `value`: a string or a signal.
/// - `icon`: a square icon button (a copy icon that turns into a tick) in
///   place of the text button.
/// - `link`: `value` is a path (`/items/K-1`); it copies the absolute URL.
/// - `label` (default "Copy"): the text, or the accessible name of the icon
///   button. `copied_label` (default "Copied"): the text after a copy.
/// - `on_copy` (optional): runs with the copied text.
///
/// The button is not there when the browser has no Clipboard API (a page on
/// plain HTTP), so there is never a button that does nothing. It confirms
/// only after the copy worked, and a screen reader hears "Copied" (an
/// `aria-live` region). A click does not go on to a clickable parent (a
/// table row).
#[component]
pub fn CopyButton(
    #[prop(into)] value: Signal<String>,
    #[prop(optional)] icon: bool,
    #[prop(optional)] link: bool,
    #[prop(default = "Copy".to_string(), into)] label: String,
    #[prop(default = "Copied".to_string(), into)] copied_label: String,
    #[prop(optional)] on_copy: Option<Callback<String>>,
) -> impl IntoView {
    if clipboard().is_none() {
        return ().into_any();
    }
    // None: idle. Some(true): copied. Some(false): the copy failed.
    let state = RwSignal::new(None::<bool>);
    let labels = StoredValue::new((label, copied_label));
    let text = move || {
        labels.with_value(|(l, c)| match state.get() {
            Some(true) => c.clone(),
            Some(false) => "Copy failed".to_string(),
            None => l.clone(),
        })
    };
    let announce = move || match state.get() {
        Some(true) => labels.with_value(|(_, c)| c.clone()),
        Some(false) => "Copy failed".to_string(),
        None => String::new(),
    };
    let on_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        let Some(clip) = clipboard() else {
            return;
        };
        let raw = value.get_untracked();
        let text = if link { absolute_url(&raw) } else { raw };
        leptos::task::spawn_local(async move {
            let ok = wasm_bindgen_futures::JsFuture::from(clip.write_text(&text))
                .await
                .is_ok();
            state.set(Some(ok));
            if ok {
                if let Some(cb) = on_copy {
                    cb.run(text);
                }
            }
            set_timeout(
                move || state.set(None),
                std::time::Duration::from_millis(1500),
            );
        });
    };
    let button = if icon {
        view! {
            <button
                type="button"
                class="cl-action-icon cl-copy"
                class:cl-copy--done=move || state.get() == Some(true)
                aria-label=move || labels.with_value(|(l, _)| l.clone())
                title=text
                on:click=on_click
            >
                {move || if state.get() == Some(true) {
                    view! { <crate::icons::IconCheck size=15 /> }.into_any()
                } else {
                    view! { <crate::icons::IconCopy size=15 /> }.into_any()
                }}
            </button>
        }
        .into_any()
    } else {
        view! {
            <button
                type="button"
                class="cl-btn cl-btn--default cl-btn--xs cl-copy"
                class:cl-copy--done=move || state.get() == Some(true)
                on:click=on_click
            >
                {text}
            </button>
        }
        .into_any()
    };
    view! {
        <span class="cl-copy-wrap">
            {button}
            <span class="cl-sr-only" role="status" aria-live="polite">{announce}</span>
        </span>
    }
    .into_any()
}

// ----------------------------------------------------------------------------
// Table
// ----------------------------------------------------------------------------

/// A styled table (Mantine `Table`). Put `<thead>`, `<tbody>`, `<tr>`,
/// `<th>` and `<td>` in it, or the helpers: [`TableRow`] (a clickable row),
/// [`SortHeader`] (a header cell that sorts) and [`TableEmpty`].
///
/// - `mono`: tabular mono cells.
/// - `fixed`: a fixed layout (`table-layout: fixed`): the columns keep their
///   widths, and long text is cut with "…".
/// - `widths`: the column widths (`vec!["22%".into(), "52px".into()]`), as a
///   `<colgroup>`. Two tables with the same widths line up.
/// - `label`: the accessible name of the table.
/// - `min_width` (optional, such as `"640px"`): the table does not get
///   narrower than this; on a small screen it scrolls sideways in its own
///   box, and the page does not.
///
/// The class `cl-num` on a `th` or `td` aligns a number to the right.
#[component]
pub fn Table(
    #[prop(optional)] mono: bool,
    #[prop(optional)] fixed: bool,
    #[prop(optional)] widths: Vec<String>,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] min_width: String,
    children: Children,
) -> impl IntoView {
    let mut class = String::from("cl-table");
    if mono {
        class.push_str(" cl-table--mono");
    }
    if fixed {
        class.push_str(" cl-table--fixed");
    }
    let cols = (!widths.is_empty()).then(|| {
        view! {
            <colgroup>
                {widths.into_iter().map(|w| view! { <col style=format!("width:{w};") /> }).collect_view()}
            </colgroup>
        }
    });
    if min_width.is_empty() {
        return view! { <table class=class aria-label=attr(label)>{cols}{children()}</table> }
            .into_any();
    }
    view! {
        <div class="cl-table-scroll">
            <table class=class aria-label=attr(label) style=format!("min-width:{min_width};")>
                {cols}
                {children()}
            </table>
        </div>
    }
    .into_any()
}

/// A table row that a person can click or open with the keyboard.
///
/// - `on_click`: runs on a click on the row, and on Enter or Space when the
///   row has focus. A click on a link, a button or a field inside the row
///   goes to that control, not to the row.
/// - `selected`: shows the row as the selected one.
/// - `label` (optional): the accessible name of the row (for example the
///   name of the item it opens).
///
/// With no `on_click` it is a plain `<tr>`.
#[component]
pub fn TableRow(
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional, into)] label: String,
    children: Children,
) -> impl IntoView {
    use leptos::wasm_bindgen::JsCast;
    let Some(cb) = on_click else {
        return view! {
            <tr class:cl-table__row--selected=move || selected.get()>{children()}</tr>
        }
        .into_any();
    };
    let on_row_click = move |ev: web_sys::MouseEvent| {
        let inner_control = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| {
                el.closest("a, button, input, select, textarea, label, [role=\"button\"]")
                    .ok()
                    .flatten()
            })
            .is_some();
        if !inner_control {
            cb.run(());
        }
    };
    let on_key = move |ev: web_sys::KeyboardEvent| {
        if ev.target() != ev.current_target() {
            return;
        }
        if ev.key() == "Enter" || ev.key() == " " {
            ev.prevent_default();
            cb.run(());
        }
    };
    view! {
        <tr
            class="cl-table__row--click"
            class:cl-table__row--selected=move || selected.get()
            tabindex="0"
            aria-label=attr(label)
            on:click=on_row_click
            on:keydown=on_key
        >
            {children()}
        </tr>
    }
    .into_any()
}

/// A header cell that sorts the table by `key`.
///
/// A click (or Enter / Space) sets `sort`: a new column starts ascending
/// (`first_desc`: descending, for times and counts), a second click flips
/// it. The cell has `aria-sort`. Sort the rows yourself from `sort`, for
/// example with `rows.sort_by(|a, b| s.order(a.name.cmp(&b.name)))`.
///
/// `align_right` aligns the header of a number column; `width` sets the
/// width of the column.
#[component]
pub fn SortHeader(
    #[prop(into)] label: String,
    #[prop(into)] key: String,
    sort: RwSignal<crate::data::SortState>,
    #[prop(optional)] first_desc: bool,
    #[prop(optional)] align_right: bool,
    #[prop(optional, into)] width: String,
) -> impl IntoView {
    use crate::data::SortDir;
    let key = StoredValue::new(key);
    let first = if first_desc {
        SortDir::Desc
    } else {
        SortDir::Asc
    };
    let dir = move || key.with_value(|k| sort.with(|s| s.dir_of(k)));
    let style = attr(if width.is_empty() {
        String::new()
    } else {
        format!("width:{width};")
    });
    view! {
        <th
            class:cl-num=align_right
            class="cl-table__sort-th"
            style=style
            aria-sort=move || key.with_value(|k| sort.with(|s| s.aria_sort(k)))
        >
            <button
                type="button"
                class="cl-table__sort"
                class:cl-table__sort--active=move || dir().is_some()
                on:click=move |_| key.with_value(|k| sort.update(|s| *s = s.toggle(k, first)))
            >
                {label}
                <span class="cl-table__sort-icon" aria-hidden="true">
                    {move || match dir() {
                        Some(SortDir::Asc) => view! { <crate::icons::IconChevron size=12 dir="up" /> }.into_any(),
                        Some(SortDir::Desc) => view! { <crate::icons::IconChevron size=12 /> }.into_any(),
                        None => view! { <crate::icons::IconChevron size=12 /> }.into_any(),
                    }}
                </span>
            </button>
        </th>
    }
}

/// A row that says the table has no rows. `colspan` is the number of
/// columns.
#[component]
pub fn TableEmpty(
    #[prop(into)] message: String,
    #[prop(default = 1)] colspan: u32,
) -> impl IntoView {
    view! {
        <tr class="cl-table__empty">
            <td colspan=colspan>{message}</td>
        </tr>
    }
}

#[cfg(test)]
mod a11y_tests {
    //! KAIROS-T-0198: labels are associated with their controls.
    //!
    //! These assert the mechanism rather than the rendered DOM — rendering a
    //! Leptos component needs a browser runtime, and the thing that broke was not
    //! the markup shape but the absence of an id to point a `for` at.

    use super::field_id;
    use std::collections::BTreeSet;

    #[test]
    fn every_field_gets_its_own_id() {
        // The bug this prevents is subtler than "no id": two fields sharing one
        // would make a label point at the wrong control, which is worse than
        // pointing at nothing because it looks correct.
        let ids: BTreeSet<String> = (0..1000).map(|_| field_id()).collect();
        assert_eq!(ids.len(), 1000, "field ids must be unique");
    }

    #[test]
    fn ids_are_valid_html_identifiers() {
        // An id starting with a digit, or containing a space, is not addressable
        // by `for=` in every browser. The prefix is there for that reason, not
        // for decoration.
        let id = field_id();
        assert!(id.starts_with("cl-field-"), "unexpected id shape: {id}");
        assert!(
            id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "id must be a plain HTML identifier: {id}"
        );
    }
}

#[cfg(test)]
mod menu_tests {
    use super::menu_key_target;

    #[test]
    fn arrows_move_and_wrap() {
        assert_eq!(menu_key_target("ArrowDown", None, 3), Some(0));
        assert_eq!(menu_key_target("ArrowDown", Some(0), 3), Some(1));
        assert_eq!(menu_key_target("ArrowDown", Some(2), 3), Some(0));
        assert_eq!(menu_key_target("ArrowUp", None, 3), Some(2));
        assert_eq!(menu_key_target("ArrowUp", Some(0), 3), Some(2));
        assert_eq!(menu_key_target("ArrowUp", Some(2), 3), Some(1));
    }

    #[test]
    fn home_end_and_other_keys() {
        assert_eq!(menu_key_target("Home", Some(2), 3), Some(0));
        assert_eq!(menu_key_target("End", Some(0), 3), Some(2));
        assert_eq!(menu_key_target("a", Some(0), 3), None);
        assert_eq!(menu_key_target("ArrowDown", None, 0), None);
        // A stale index past the end wraps to a real item.
        assert_eq!(menu_key_target("ArrowUp", Some(9), 3), Some(2));
    }
}
