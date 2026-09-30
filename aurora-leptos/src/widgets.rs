//! Aurora **widgets** — generic, higher-level data-display building blocks
//! that control-plane UIs compose from the core primitives: [`Meter`],
//! [`Banner`], [`HealthPill`] and [`BuildStatusBadge`].
//!
//! Apps supply their own state labels, colors, and tooltips as data — this
//! module ships no app-specific vocab, palette, or branding. The data
//! components (StatTile, SegmentedBar, LogView, ...) are in
//! [`crate::data`].
//!
//! 0.4 removed `NodeReadiness`, `InputTable`, `StaleInputsBanner`,
//! `StateCounts`, the `Input` model and `format_ago` / `is_stale` /
//! `freshness_pct`: no product used them. Use [`crate::data::RelativeTime`]
//! and [`crate::data::format_relative`] for "time ago", and
//! [`crate::data::SegmentedBar`] for counts per state.

use leptos::prelude::*;

use crate::components::{Pill, Tooltip};
use crate::tokens::token;

// ----------------------------------------------------------------------------
// Generic primitives
// ----------------------------------------------------------------------------

/// A thin horizontal progress / freshness bar. `value` is 0–100 (a number
/// or a signal); `color` defaults to the ok/green token. `label` (optional)
/// makes it a `role="meter"` with that accessible name.
///
/// For a bar in several parts, use
/// [`SegmentedBar`](crate::data::SegmentedBar).
#[component]
pub fn Meter(
    #[prop(into)] value: Signal<f64>,
    #[prop(optional, into)] color: String,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    let color = if color.is_empty() {
        token::OK.to_string()
    } else {
        color
    };
    let pct = move || value.get().clamp(0.0, 100.0);
    let named = !label.is_empty();
    view! {
        <div
            class="cl-meter"
            role=named.then_some("meter")
            aria-label=named.then_some(label)
            aria-valuemin=named.then_some("0")
            aria-valuemax=named.then_some("100")
            aria-valuenow=move || named.then(|| format!("{:.0}", pct()))
        >
            <div class="cl-meter__fill" style=move || format!("width:{}%;background:{color};", pct())></div>
        </div>
    }
}

/// A tinted callout banner. `color` sets the accent (defaults to gold/warn);
/// `icon` defaults to a warning triangle.
#[component]
pub fn Banner(
    #[prop(optional, into)] color: String,
    #[prop(optional, into)] icon: String,
    children: Children,
) -> impl IntoView {
    let color = if color.is_empty() {
        token::GOLD.to_string()
    } else {
        color
    };
    let icon = if icon.is_empty() {
        "⚠".to_string()
    } else {
        icon
    };
    view! {
        <div class="cl-banner" style=format!("--banner-color:{color};")>
            <span class="cl-banner__icon">{icon}</span>
            <span class="cl-banner__text">{children()}</span>
        </div>
    }
}

/// A status/health pill with an optional explanatory tooltip. The caller supplies
/// the display `label`, `color`, and `tip` (no built-in vocab).
#[component]
pub fn HealthPill(
    #[prop(into)] label: String,
    #[prop(into)] color: String,
    #[prop(optional, into)] tip: String,
) -> impl IntoView {
    if tip.is_empty() {
        view! { <Pill color=color>{label}</Pill> }.into_any()
    } else {
        view! { <Tooltip label=tip><Pill color=color>{label}</Pill></Tooltip> }.into_any()
    }
}

// ----------------------------------------------------------------------------
// BuildStatusBadge — generic CI/CD build state (success/building/failed/pending)
// ----------------------------------------------------------------------------

fn build_status_color(status: &str) -> &'static str {
    match status {
        "success" => token::OK,
        "failed" => token::BAD,
        "building" => token::ICE,
        "pending" => token::MUTED,
        _ => token::MUTED,
    }
}

#[component]
pub fn BuildStatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let color = build_status_color(&status).to_string();
    view! { <Pill color=color>{status}</Pill> }
}
