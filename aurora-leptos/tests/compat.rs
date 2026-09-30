//! The call shapes the products use today still compile (COLLIERY-T-1834).
//!
//! Each function copies the props that Kairos, Cloacina, Weir, Brokkr or the
//! Hlin pack pass to an Aurora component (from a read of their code). The
//! functions are only compiled, not run: a `view!` needs a browser to render.
//! If a change to a component breaks one of these shapes, this file stops
//! compiling.

use aurora_leptos::components::*;
use aurora_leptos::widgets::*;
use leptos::prelude::*;

#[allow(dead_code)]
fn inputs_as_the_products_call_them() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let pick = RwSignal::new(String::from("a"));
    let on = RwSignal::new(false);
    let n = RwSignal::new(1.0_f64);
    let options = vec!["a".to_string(), "b".to_string()];
    let offset = 0_i64;
    let busy = RwSignal::new(false);
    let archived = true;
    view! {
        // Button: variant, size, bad, disabled (a literal, an expression, a
        // precomputed bool), on_click.
        <Button variant="default" size="xs" disabled=true>"Prev"</Button>
        <Button variant="subtle" size="xs" bad=true disabled={offset == 0}
            on_click=Callback::new(|_| {})>"Next"</Button>
        <Button disabled=archived>"Archive"</Button>
        <Button disabled=busy.get()>"Save"</Button>
        <Button variant="primary">"Primary is filled"</Button>
        <TextInput label="Name" placeholder="nightly" value=name error="Required" />
        <TextInput value=name />
        <PasswordInput label="Token" placeholder="paste" value=name />
        <NumberInput label="Limit" value=n />
        <Textarea label="Notes" placeholder="…" rows=3 value=name />
        <Select label="Board" options=options.clone() value=pick />
        <Select label="Board" options value=pick />
        <Switch checked=on label="Live" />
        <CopyButton value="exec_7f3a01" />
        <CopyButton value=String::from("key") />
        <Table mono=true><tbody><tr><td>"x"</td></tr></tbody></Table>
        <Table><tbody></tbody></Table>
        <Menu label="Actions"><MenuItem on_click=Callback::new(|_| {})>"Run"</MenuItem></Menu>
        <Meter value=40.0 color="var(--ok)" />
        <HealthPill label="live" color="var(--ok)" tip="Connected." />
        <Tooltip label="Why"><span>"?"</span></Tooltip>
    }
}

#[allow(dead_code)]
fn new_props() -> impl IntoView {
    let v = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    view! {
        <Button disabled=move || busy.get() loading=busy loading_label="Saving…"
            button_type="submit">"Save"</Button>
        <Button href="/workflows/new" variant="filled">"New"</Button>
        <TextInput label="Email" value=v input_type="email" name="email"
            autocomplete="username" required=true spellcheck=false
            disabled=busy on_input=Callback::new(|_s: String| {})
            on_change=Callback::new(|_s: String| {}) />
        <PasswordInput label="Password" value=v name="password"
            autocomplete="current-password" required=true />
        <Select label="Doc" value=v placeholder="(none)"
            option_pairs=vec![("1".to_string(), "One".to_string())]
            on_change=Callback::new(|_s: String| {}) disabled=busy />
        <Switch checked=busy disabled=true on_change=Callback::new(|_b: bool| {}) />
    }
}

#[test]
fn call_shapes_compile() {
    // Referencing the functions makes the compiler check them.
    let _ = inputs_as_the_products_call_them;
    let _ = new_props;
}
