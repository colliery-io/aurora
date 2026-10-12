//! Aurora form controls added in AURORA-T-0013 item 22: [`Checkbox`],
//! [`FileInput`], [`Slider`] and [`Combobox`].
//!
//! Each is a native control where HTML has one (a checkbox, a file input, a
//! range input), so the keyboard and screen readers work as they do anywhere.
//! They take the field props of [`crate::components::TextInput`]: `label`,
//! `description`, `error`, `id`, `aria_label`, `disabled`.
//!
//! Everything here is also re-exported from [`crate::components`].

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::components::{
    attr, control_id, describedby, field_error, field_help, field_label, menu_key_target,
};

// ============================================================================
// Pure logic
// ============================================================================

/// The indexes of the choices of a [`Combobox`] that match the typed text:
/// each choice whose label contains it, with no regard to case. An empty
/// text matches every choice.
pub fn combobox_matches(labels: &[String], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    labels
        .iter()
        .enumerate()
        .filter(|(_, l)| query.is_empty() || l.to_lowercase().contains(&query))
        .map(|(i, _)| i)
        .collect()
}

// ============================================================================
// Checkbox
// ============================================================================

/// A checkbox, bound to `checked`.
///
/// - `label`: the text after the box; a click on it toggles the box.
/// - `indeterminate` (optional): shows a dash ("some of them"), for a box
///   that checks a list. A signal or a closure.
/// - `description`, `error`, `disabled`, `required`, `name`, `value`, `id`,
///   `aria_label`, `described_by`: as on a text field.
/// - `on_change`: runs with the new state.
#[component]
pub fn Checkbox(
    checked: RwSignal<bool>,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] indeterminate: Signal<bool>,
    #[prop(optional, into)] description: Signal<String>,
    #[prop(optional, into)] described_by: String,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] value: String,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional)] on_change: Option<Callback<bool>>,
) -> impl IntoView {
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let has_error = move || error.with(|e| !e.is_empty());
    let described = describedby(&id, description, described_by, error);
    let has_label = !label.is_empty();
    view! {
        <div class="cl-field cl-check-field">
            <label class="cl-check" class:cl-check--disabled=move || disabled.get()>
                <input
                    type="checkbox"
                    class="cl-checkbox"
                    class:cl-input--error=has_error
                    id=id.clone()
                    name=attr(name)
                    value=attr(value)
                    aria-label=attr(aria_label)
                    required=required
                    aria-invalid=move || has_error().then_some("true")
                    aria-describedby=described
                    disabled=move || disabled.get()
                    prop:checked=move || checked.get()
                    prop:indeterminate=move || indeterminate.get()
                    on:change=move |e| {
                        let on = event_target_checked(&e);
                        checked.set(on);
                        if let Some(cb) = on_change { cb.run(on); }
                    }
                />
                {has_label.then(|| view! {
                    <span class="cl-check__label">
                        {label}
                        {required.then(|| view! { <span class="cl-field__required" aria-hidden="true">" *"</span> })}
                    </span>
                })}
            </label>
            {field_help(description, format!("{id}-help"))}
            {field_error(error, err_id)}
        </div>
    }
}

// ============================================================================
// FileInput
// ============================================================================

/// A file picker: a button that opens the browser's file dialog, and the
/// names of the files chosen.
///
/// - `accept`: the file types (`".srt,.vtt"`, `"image/*"`).
/// - `multiple`: more than one file.
/// - `button_label`: the text of the button (default "Choose file").
/// - `on_change`: runs with the chosen files.
/// - `label`, `description`, `error`, `disabled`, `required`, `name`, `id`,
///   `aria_label`, `described_by`: as on a text field.
///
/// The `<input type="file">` is in the page and takes focus; only its look is
/// replaced. Its label is `label`, or `aria_label`, or the button text.
#[component]
pub fn FileInput(
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] accept: String,
    #[prop(optional)] multiple: bool,
    #[prop(default = "Choose file".to_string(), into)] button_label: String,
    #[prop(default = "No file chosen".to_string(), into)] empty_label: String,
    #[prop(optional, into)] description: Signal<String>,
    #[prop(optional, into)] described_by: String,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional)] on_change: Option<Callback<Vec<web_sys::File>>>,
) -> impl IntoView {
    let id = control_id(id);
    let err_id = format!("{id}-error");
    let has_error = move || error.with(|e| !e.is_empty());
    let described = describedby(&id, description, described_by, error);
    let names = RwSignal::new(Vec::<String>::new());
    let aria_label = if aria_label.is_empty() && label.is_empty() {
        button_label.clone()
    } else {
        aria_label
    };
    view! {
        <div class="cl-field">
            {field_label(label, id.clone(), required)}
            <div class="cl-file" class:cl-file--disabled=move || disabled.get()>
                <input
                    type="file"
                    class="cl-file__input"
                    id=id.clone()
                    name=attr(name)
                    accept=attr(accept)
                    multiple=multiple
                    required=required
                    aria-label=attr(aria_label)
                    aria-invalid=move || has_error().then_some("true")
                    aria-describedby=described
                    disabled=move || disabled.get()
                    on:change=move |e| {
                        let files: Vec<web_sys::File> = e
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                            .and_then(|i| i.files())
                            .map(|list| (0..list.length()).filter_map(|i| list.get(i)).collect())
                            .unwrap_or_default();
                        names.set(files.iter().map(|f| f.name()).collect());
                        if let Some(cb) = on_change { cb.run(files); }
                    }
                />
                <span class="cl-btn cl-btn--default cl-btn--xs cl-file__button" aria-hidden="true">
                    {button_label}
                </span>
                <span class="cl-file__names" aria-hidden="true">
                    {move || {
                        let n = names.get();
                        if n.is_empty() { empty_label.clone() } else { n.join(", ") }
                    }}
                </span>
            </div>
            {field_help(description, format!("{id}-help"))}
            {field_error(error, err_id)}
        </div>
    }
}

// ============================================================================
// Slider
// ============================================================================

/// A slider (a native range input), bound to `value`.
///
/// - `min` (default 0), `max` (default 100), `step` (default 1).
/// - `show_value` (default true): the value at the end of the label line.
///   `format` (optional) makes its text ("40%", "2.5 MB/s"); it is also the
///   `aria-valuetext`.
/// - `label`, `description`, `disabled`, `name`, `id`, `aria_label`,
///   `described_by`: as on a text field.
/// - `on_input`: runs on each move. `on_change`: runs when the person lets go.
#[component]
pub fn Slider(
    value: RwSignal<f64>,
    #[prop(default = 0.0)] min: f64,
    #[prop(default = 100.0)] max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(default = true)] show_value: bool,
    #[prop(optional)] format: Option<Callback<f64, String>>,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] description: Signal<String>,
    #[prop(optional, into)] described_by: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional)] on_input: Option<Callback<f64>>,
    #[prop(optional)] on_change: Option<Callback<f64>>,
) -> impl IntoView {
    let id = control_id(id);
    let described = describedby(&id, description, described_by, Signal::default());
    let text = move || match format {
        Some(f) => f.run(value.get()),
        None => {
            let v = value.get();
            if v.fract() == 0.0 {
                format!("{}", v as i64)
            } else {
                format!("{v}")
            }
        }
    };
    let has_label = !label.is_empty();
    let fill = move || {
        let span = (max - min).abs().max(f64::EPSILON);
        let pct = ((value.get() - min) / span * 100.0).clamp(0.0, 100.0);
        format!("--cl-slider-fill:{pct}%;")
    };
    view! {
        <div class="cl-field cl-slider-field">
            {(has_label || show_value).then(|| view! {
                <div class="cl-slider__head">
                    {has_label.then(|| view! { <label class="cl-field__label" for=id.clone()>{label}</label> })}
                    {show_value.then(|| view! { <span class="cl-slider__value cl-tnum" aria-hidden="true">{text}</span> })}
                </div>
            })}
            <input
                type="range"
                class="cl-slider"
                id=id.clone()
                name=attr(name)
                min=min
                max=max
                step=step
                style=fill
                aria-label=attr(aria_label)
                aria-valuetext=move || format.map(|f| f.run(value.get()))
                aria-describedby=described
                disabled=move || disabled.get()
                prop:value=move || value.get().to_string()
                on:input=move |e| {
                    if let Ok(v) = event_target_value(&e).parse::<f64>() {
                        value.set(v);
                        if let Some(cb) = on_input { cb.run(v); }
                    }
                }
                on:change=move |e| {
                    if let Ok(v) = event_target_value(&e).parse::<f64>() {
                        if let Some(cb) = on_change { cb.run(v); }
                    }
                }
            />
            {field_help(description, format!("{id}-help"))}
        </div>
    }
}

// ============================================================================
// Combobox
// ============================================================================

/// A text field with a list of choices that narrows as the person types (the
/// ARIA combobox pattern). `value` holds the value of the chosen choice.
///
/// - `options`: `(value, label)` choices. A `Vec`, a signal or a closure.
/// - `placeholder`, `empty_text` (default "No matches").
/// - `label`, `description`, `error`, `disabled`, `required`, `name`, `id`,
///   `aria_label`, `described_by`: as on a text field.
/// - `on_change`: runs with the value of a choice when the person picks it.
///
/// ArrowDown and ArrowUp open the list and move through it; Enter picks;
/// Escape closes the list, and a second Escape clears the text. When the
/// field loses focus the text goes back to the label of the chosen value.
#[component]
pub fn Combobox(
    value: RwSignal<String>,
    #[prop(into)] options: Signal<Vec<(String, String)>>,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(default = "No matches".to_string(), into)] empty_text: String,
    #[prop(optional, into)] description: Signal<String>,
    #[prop(optional, into)] described_by: String,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] id: String,
    #[prop(optional, into)] aria_label: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    let id = control_id(id);
    let list_id = format!("{id}-list");
    let err_id = format!("{id}-error");
    let has_error = move || error.with(|e| !e.is_empty());
    let described = describedby(&id, description, described_by, error);

    let label_of =
        move |v: &str| options.with(|os| os.iter().find(|(ov, _)| ov == v).map(|(_, l)| l.clone()));
    // The text in the field: the label of the chosen value until the person types.
    let query = RwSignal::new(label_of(&value.get_untracked()).unwrap_or_default());
    let typing = RwSignal::new(false);
    let open = RwSignal::new(false);
    let active = RwSignal::new(None::<usize>);

    // When the value changes from outside, the text follows.
    Effect::new(move |_| {
        let v = value.get();
        if !typing.get_untracked() {
            query.set(label_of(&v).unwrap_or_default());
        }
    });

    let matches = Memo::new(move |_| {
        let q = if typing.get() {
            query.get()
        } else {
            String::new()
        };
        options.with(|os| {
            let labels: Vec<String> = os.iter().map(|(_, l)| l.clone()).collect();
            combobox_matches(&labels, &q)
                .into_iter()
                .map(|i| os[i].clone())
                .collect::<Vec<_>>()
        })
    });
    let option_id = {
        let list_id = list_id.clone();
        move |i: usize| format!("{list_id}-{i}")
    };

    let choose = move |v: String| {
        let l = label_of(&v).unwrap_or_default();
        value.set(v.clone());
        typing.set(false);
        query.set(l);
        open.set(false);
        active.set(None);
        if let Some(cb) = on_change {
            cb.run(v);
        }
    };
    let reset = move || {
        typing.set(false);
        query.set(label_of(&value.get_untracked()).unwrap_or_default());
        open.set(false);
        active.set(None);
    };

    let active_id = {
        let option_id = option_id.clone();
        move || {
            open.get()
                .then(|| active.get())
                .flatten()
                .map(&option_id)
        }
    };

    let list = {
        let option_id = option_id.clone();
        let list_id = list_id.clone();
        move || {
            let items = matches.get();
            let option_id = option_id.clone();
            open.get().then(|| {
                let body = if items.is_empty() {
                    view! { <li class="cl-combobox__empty" role="presentation">{empty_text.clone()}</li> }
                        .into_any()
                } else {
                    items
                        .into_iter()
                        .enumerate()
                        .map(|(i, (v, l))| {
                            let selected = value.with_untracked(|cur| *cur == v);
                            view! {
                                <li
                                    class="cl-combobox__option"
                                    class:cl-combobox__option--active=move || active.get() == Some(i)
                                    id=option_id(i)
                                    role="option"
                                    aria-selected=if selected { "true" } else { "false" }
                                    on:mousedown=move |ev: web_sys::MouseEvent| ev.prevent_default()
                                    on:click=move |_| choose(v.clone())
                                >
                                    {l}
                                </li>
                            }
                        })
                        .collect_view()
                        .into_any()
                };
                view! {
                    <ul class="cl-combobox__list" id=list_id.clone() role="listbox">{body}</ul>
                }
            })
        }
    };

    view! {
        <div class="cl-field">
            {field_label(label, id.clone(), required)}
            <div class="cl-combobox">
                <input
                    class="cl-input cl-combobox__input"
                    class:cl-input--error=has_error
                    id=id.clone()
                    type="text"
                    role="combobox"
                    autocomplete="off"
                    aria-autocomplete="list"
                    aria-expanded=move || if open.get() { "true" } else { "false" }
                    aria-controls=list_id.clone()
                    aria-activedescendant=active_id
                    placeholder=attr(placeholder)
                    name=attr(name)
                    aria-label=attr(aria_label)
                    required=required
                    aria-invalid=move || has_error().then_some("true")
                    aria-describedby=described
                    disabled=move || disabled.get()
                    prop:value=move || query.get()
                    on:input=move |e| {
                        query.set(event_target_value(&e));
                        typing.set(true);
                        open.set(true);
                        active.set(if matches.with_untracked(|m| m.is_empty()) { None } else { Some(0) });
                    }
                    on:click=move |_| open.set(true)
                    on:blur=move |_| reset()
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        let count = matches.with_untracked(|m| m.len());
                        match ev.key().as_str() {
                            "ArrowDown" | "ArrowUp" => {
                                ev.prevent_default();
                                if !open.get_untracked() {
                                    open.set(true);
                                }
                                active.set(menu_key_target(&ev.key(), active.get_untracked(), count));
                            }
                            "Enter" => {
                                if open.get_untracked() {
                                    if let Some(i) = active.get_untracked() {
                                        ev.prevent_default();
                                        if let Some((v, _)) = matches.with_untracked(|m| m.get(i).cloned()) {
                                            choose(v);
                                        }
                                    }
                                }
                            }
                            "Escape" => {
                                if open.get_untracked() {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    open.set(false);
                                    active.set(None);
                                } else if !query.get_untracked().is_empty() {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    query.set(String::new());
                                    typing.set(true);
                                }
                            }
                            _ => {}
                        }
                    }
                />
                {list}
            </div>
            {field_help(description, format!("{id}-help"))}
            {field_error(error, err_id)}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::combobox_matches;

    fn labels(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn empty_text_matches_every_choice() {
        assert_eq!(combobox_matches(&labels(&["a", "b"]), ""), vec![0, 1]);
        assert_eq!(combobox_matches(&labels(&["a", "b"]), "  "), vec![0, 1]);
    }

    #[test]
    fn matches_a_part_with_no_regard_to_case() {
        let l = labels(&["Movies", "TV shows", "Audiobooks"]);
        assert_eq!(combobox_matches(&l, "o"), vec![0, 1, 2]);
        assert_eq!(combobox_matches(&l, "SHOW"), vec![1]);
        assert_eq!(combobox_matches(&l, "zzz"), Vec::<usize>::new());
    }
}
