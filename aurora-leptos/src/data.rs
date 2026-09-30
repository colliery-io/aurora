//! Aurora data components — small pieces that show data on a page.
//!
//! - [`StatTile`]: a label, a big value, a unit, a delta or a sub line, and a
//!   slot for a [`Sparkline`].
//! - [`Sparkline`]: a tiny line or bar chart in `currentColor`.
//! - [`SegmentedBar`]: one bar in parts (ok / warning / failed, done / active
//!   / to do), with an optional legend.
//! - [`DetailList`] + [`KeyValue`]: label and value rows.
//! - [`SectionLabel`]: a mono uppercase heading with a count and an action.
//! - [`CodeBlock`] and [`LogView`]: text in a mono block that scrolls; the log
//!   has a time and a level on each line, follows the tail, and can copy.
//! - [`FeedList`] + [`FeedRow`]: an activity or run feed.
//! - [`Pagination`]: Previous / Next, "41–60 of 212", rows per page.
//! - [`RelativeTime`]: "3m ago" in a `<time>`, with the full time on hover.
//!   All of them share one clock ([`use_now`]).
//! - [`LiveIndicator`]: live / connecting / offline for a stream.
//!
//! The pure logic (relative time, durations, page ranges, sort state,
//! sparkline geometry, bar widths) is plain Rust with unit tests at the end of
//! this file. Everything here is re-exported from [`crate::components`].

use std::cell::RefCell;
use std::cmp::Ordering;

use leptos::html;
use leptos::prelude::*;

use crate::components::{Button, CopyButton};
use crate::icons::IconChevron;
use crate::tokens::{fg_for, fill_for, status_color, token};

// ============================================================================
// Pure logic
// ============================================================================

/// "3m ago" for a time `delta_ms` in the past (a negative delta is in the
/// future: "in 3m"). Under 5 seconds is "just now". `NaN` is "—".
///
/// Steps: seconds, minutes, hours, days, then months (30 days) and years
/// (365 days).
pub fn format_relative(delta_ms: f64) -> String {
    if !delta_ms.is_finite() {
        return "—".into();
    }
    let future = delta_ms < 0.0;
    let s = (delta_ms.abs() / 1000.0).floor() as u64;
    if s < 5 {
        return "just now".into();
    }
    let span = if s < 60 {
        format!("{s}s")
    } else if s < 3_600 {
        format!("{}m", s / 60)
    } else if s < 86_400 {
        format!("{}h", s / 3_600)
    } else if s < 30 * 86_400 {
        format!("{}d", s / 86_400)
    } else if s < 365 * 86_400 {
        format!("{}mo", s / (30 * 86_400))
    } else {
        format!("{}y", s / (365 * 86_400))
    };
    if future {
        format!("in {span}")
    } else {
        format!("{span} ago")
    }
}

/// A duration for people: `850ms`, `4.2s`, `3m 05s`, `2h 04m`, `3d 2h`.
/// A negative or `NaN` duration is "—".
pub fn format_duration(ms: f64) -> String {
    if !ms.is_finite() || ms < 0.0 {
        return "—".into();
    }
    if ms < 1_000.0 {
        return format!("{}ms", ms.round() as u64);
    }
    let secs = ms / 1_000.0;
    if secs < 60.0 {
        // 59.97s would round to "60.0s"; show it as the next step instead.
        let tenths = (secs * 10.0).round() / 10.0;
        if tenths < 60.0 {
            return format!("{tenths:.1}s");
        }
    }
    let total = secs.round() as u64;
    let (d, h, m, s) = (
        total / 86_400,
        (total % 86_400) / 3_600,
        (total % 3_600) / 60,
        total % 60,
    );
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m {s:02}s")
    }
}

/// The part of a list that one page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageRange {
    /// The first item on the page, from 1. Zero when the list is empty.
    pub first: usize,
    /// The last item on the page. Zero when the list is empty.
    pub last: usize,
    pub total: usize,
    /// The offset of the page after the clamp (see [`page_range`]).
    pub offset: usize,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_offset: usize,
    pub next_offset: usize,
}

/// The page at `offset` with `limit` items per page, in a list of `total`.
///
/// An offset past the end moves to the start of the last page. A `limit` of
/// zero counts as one.
pub fn page_range(offset: usize, limit: usize, total: usize) -> PageRange {
    let limit = limit.max(1);
    let offset = if total == 0 {
        0
    } else if offset >= total {
        ((total - 1) / limit) * limit
    } else {
        offset
    };
    let last = (offset + limit).min(total);
    PageRange {
        first: if total == 0 { 0 } else { offset + 1 },
        last,
        total,
        offset,
        has_prev: offset > 0,
        has_next: last < total,
        prev_offset: offset.saturating_sub(limit),
        next_offset: if last < total { offset + limit } else { offset },
    }
}

/// "41–60 of 212" (an en dash), or "0 of 0" for an empty list.
pub fn page_range_label(r: &PageRange) -> String {
    if r.total == 0 {
        "0 of 0".into()
    } else {
        format!("{}–{} of {}", r.first, r.last, r.total)
    }
}

/// The offset after the page size changes to `new_limit`: the start of the
/// new page that holds the first item of the old page.
pub fn offset_for_limit(offset: usize, new_limit: usize) -> usize {
    let new_limit = new_limit.max(1);
    (offset / new_limit) * new_limit
}

/// The direction of a sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    /// The other direction.
    pub fn flip(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }

    /// `ord` in this direction: the same for `Asc`, reversed for `Desc`.
    pub fn apply(self, ord: Ordering) -> Ordering {
        match self {
            SortDir::Asc => ord,
            SortDir::Desc => ord.reverse(),
        }
    }
}

/// Which column a table is sorted by, and in which direction.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SortState {
    /// The column key; `None` when the table is not sorted.
    pub key: Option<String>,
    pub dir: SortDir,
}

impl SortState {
    /// Sorted by `key` in `dir`.
    pub fn by(key: impl Into<String>, dir: SortDir) -> Self {
        Self {
            key: Some(key.into()),
            dir,
        }
    }

    /// The state after a click on the header of `key`.
    ///
    /// A new column starts in `first` (ascending for names, descending is
    /// usual for times and counts). A click on the sorted column flips it.
    pub fn toggle(&self, key: &str, first: SortDir) -> Self {
        if self.key.as_deref() == Some(key) {
            Self::by(key, self.dir.flip())
        } else {
            Self::by(key, first)
        }
    }

    /// The direction of `key`, or `None` when the table is sorted by another
    /// column.
    pub fn dir_of(&self, key: &str) -> Option<SortDir> {
        (self.key.as_deref() == Some(key)).then_some(self.dir)
    }

    /// The `aria-sort` value for the header of `key`.
    pub fn aria_sort(&self, key: &str) -> &'static str {
        match self.dir_of(key) {
            Some(SortDir::Asc) => "ascending",
            Some(SortDir::Desc) => "descending",
            None => "none",
        }
    }

    /// `ord` in the direction of this state (a helper for `sort_by`).
    pub fn order(&self, ord: Ordering) -> Ordering {
        self.dir.apply(ord)
    }
}

/// The points of a sparkline in a `width` × `height` box.
///
/// The first value is at x = 0 and the last at x = `width`. The y axis goes
/// from the smallest value (bottom) to the largest (top); with `zero_base`
/// the bottom is 0 when all values are positive. A flat series draws in the
/// middle. A 1px margin at the top and the bottom keeps the stroke inside.
pub fn spark_points(values: &[f64], width: f64, height: f64, zero_base: bool) -> Vec<(f64, f64)> {
    let vals: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if vals.is_empty() {
        return Vec::new();
    }
    let (lo, hi) = spark_bounds(&vals, zero_base);
    let pad = 1.0_f64.min(height / 4.0);
    let inner = height - 2.0 * pad;
    let n = vals.len();
    vals.iter()
        .enumerate()
        .map(|(i, v)| {
            let x = if n == 1 {
                width / 2.0
            } else {
                i as f64 / (n - 1) as f64 * width
            };
            let y = if hi > lo {
                pad + inner - (v - lo) / (hi - lo) * inner
            } else {
                height / 2.0
            };
            (round2(x), round2(y))
        })
        .collect()
}

fn spark_bounds(vals: &[f64], zero_base: bool) -> (f64, f64) {
    let mut lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let mut hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if zero_base {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    (lo, hi)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// An SVG path (`M x y L x y ...`) through `points`. One point draws a short
/// flat line so that it shows.
pub fn spark_line_path(points: &[(f64, f64)]) -> String {
    match points {
        [] => String::new(),
        [(x, y)] => format!("M{} {}L{} {}", x - 2.0, y, x + 2.0, y),
        _ => {
            let mut d = String::new();
            for (i, (x, y)) in points.iter().enumerate() {
                d.push_str(if i == 0 { "M" } else { "L" });
                d.push_str(&format!("{x} {y}"));
            }
            d
        }
    }
}

/// The closed SVG path under a sparkline line, down to the bottom (`height`).
pub fn spark_area_path(points: &[(f64, f64)], height: f64) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let first = points[0].0;
    let last = points[points.len() - 1].0;
    format!(
        "{}L{last} {height}L{first} {height}Z",
        spark_line_path(points)
    )
}

/// The bars of a bar sparkline: `(x, y, width, height)` for each value, with
/// `gap` px between bars. A bar is at least 1px high so that a small value
/// shows; a zero or missing value has no height.
pub fn spark_bars(
    values: &[f64],
    width: f64,
    height: f64,
    gap: f64,
    zero_base: bool,
) -> Vec<(f64, f64, f64, f64)> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return Vec::new();
    }
    let (lo, hi) = spark_bounds(&finite, zero_base);
    let slot = width / n as f64;
    let bar_w = (slot - gap).max(1.0);
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let h = if !v.is_finite() {
                0.0
            } else if hi > lo {
                let h = (v - lo) / (hi - lo) * height;
                if *v != 0.0 {
                    h.max(1.0)
                } else {
                    h
                }
            } else if *v != 0.0 {
                height
            } else {
                0.0
            };
            let x = i as f64 * slot + (slot - bar_w) / 2.0;
            (round2(x), round2(height - h), round2(bar_w), round2(h))
        })
        .collect()
}

/// The width of each segment in percent. Negative and `NaN` values count as
/// zero. When all values are zero, all widths are zero.
pub fn segment_widths(values: &[f64]) -> Vec<f64> {
    let clean: Vec<f64> = values
        .iter()
        .map(|v| if v.is_finite() && *v > 0.0 { *v } else { 0.0 })
        .collect();
    let total: f64 = clean.iter().sum();
    if total <= 0.0 {
        return vec![0.0; clean.len()];
    }
    clean.iter().map(|v| v / total * 100.0).collect()
}

/// The text of `lines` as the copy button of a [`LogView`] puts it on the
/// clipboard: time, level and text, one line each.
pub fn log_text(lines: &[LogLine]) -> String {
    lines
        .iter()
        .map(|l| {
            let mut parts = Vec::new();
            if let Some(t) = &l.time {
                parts.push(t.as_str());
            }
            if let Some(lv) = &l.level {
                parts.push(lv.as_str());
            }
            parts.push(l.text.as_str());
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// The shared clock
// ============================================================================

thread_local! {
    static NOW: RefCell<Option<ArcRwSignal<f64>>> = const { RefCell::new(None) };
}

/// How often the shared clock ticks.
pub const CLOCK_TICK_MS: u64 = 1_000;

/// The time now (ms since the epoch), as a signal that changes once a second.
///
/// All callers share one signal and one timer: the first call starts it, and
/// it runs for the life of the page. [`RelativeTime`] uses it, so "12s ago"
/// moves on between two fetches of the data.
pub fn use_now() -> Signal<f64> {
    let sig = NOW.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(|| {
                let now = ArcRwSignal::new(js_sys::Date::now());
                let tick = now.clone();
                let _ = set_interval_with_handle(
                    move || tick.set(js_sys::Date::now()),
                    std::time::Duration::from_millis(CLOCK_TICK_MS),
                );
                now
            })
            .clone()
    });
    sig.into()
}

/// Milliseconds since the epoch for an RFC 3339 / ISO 8601 time string, or
/// `None` when the browser cannot read it.
pub fn parse_timestamp(s: &str) -> Option<f64> {
    let ms = js_sys::Date::parse(s);
    ms.is_finite().then_some(ms)
}

// ============================================================================
// RelativeTime
// ============================================================================

/// A time as "3m ago", in a `<time datetime="...">`. The full local time shows
/// as a tooltip (`title`). It moves on once a second with the shared clock.
///
/// Give the time as `at` (ms since the epoch) or as `iso` (a string such as
/// `2026-09-30T12:04:53Z`). With neither, or a time the browser cannot read,
/// it shows `fallback` ("—").
///
/// For a duration ("4.2s", "3m 05s") use [`format_duration`].
#[component]
pub fn RelativeTime(
    #[prop(optional, into)] at: MaybeProp<f64>,
    #[prop(optional, into)] iso: MaybeProp<String>,
    #[prop(default = "—".to_string(), into)] fallback: String,
) -> impl IntoView {
    let now = use_now();
    let ms = Memo::new(move |_| {
        at.get()
            .filter(|v| v.is_finite())
            .or_else(|| iso.get().and_then(|s| parse_timestamp(&s)))
    });
    let fallback = StoredValue::new(fallback);
    let label = Memo::new(move |_| match ms.get() {
        Some(t) => format_relative(now.get() - t),
        None => fallback.get_value(),
    });
    let datetime = move || {
        ms.get().map(|t| {
            String::from(
                js_sys::Date::new(&leptos::wasm_bindgen::JsValue::from_f64(t)).to_iso_string(),
            )
        })
    };
    let title = move || {
        ms.get().map(|t| {
            String::from(
                js_sys::Date::new(&leptos::wasm_bindgen::JsValue::from_f64(t))
                    .to_locale_string("default", &leptos::wasm_bindgen::JsValue::UNDEFINED),
            )
        })
    };
    view! {
        <time class="cl-reltime" datetime=datetime title=title>{label}</time>
    }
}

// ============================================================================
// LiveIndicator
// ============================================================================

/// The state of a live stream (a WebSocket, a poll loop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LiveState {
    /// Updates come in.
    Live,
    /// The connection starts or comes back.
    #[default]
    Connecting,
    /// No updates come in.
    Offline,
}

impl LiveState {
    /// The hue token: ok, gold, muted.
    pub fn color(self) -> &'static str {
        match self {
            LiveState::Live => token::OK,
            LiveState::Connecting => token::GOLD,
            LiveState::Offline => token::MUTED,
        }
    }

    fn class(self) -> &'static str {
        match self {
            LiveState::Live => "live",
            LiveState::Connecting => "connecting",
            LiveState::Offline => "offline",
        }
    }
}

/// A dot and a word for the state of a live stream: live, connecting or
/// offline. The dot pulses only while live, and not at all when the system
/// asks for less motion.
///
/// - `state`: a signal of [`LiveState`].
/// - `live_label` / `connecting_label` / `offline_label`: the words
///   (defaults "Live", "Connecting", "Offline").
/// - `compact`: the dot only; the word goes to the tooltip and the
///   accessible name.
///
/// It is a `role="status"` region, so a screen reader says the new state.
#[component]
pub fn LiveIndicator(
    #[prop(into)] state: Signal<LiveState>,
    #[prop(default = "Live".to_string(), into)] live_label: String,
    #[prop(default = "Connecting".to_string(), into)] connecting_label: String,
    #[prop(default = "Offline".to_string(), into)] offline_label: String,
    #[prop(optional)] compact: bool,
) -> impl IntoView {
    let labels = StoredValue::new((live_label, connecting_label, offline_label));
    let label = move || {
        labels.with_value(|(l, c, o)| match state.get() {
            LiveState::Live => l.clone(),
            LiveState::Connecting => c.clone(),
            LiveState::Offline => o.clone(),
        })
    };
    let class = move || {
        let mut c = format!("cl-live cl-live--{}", state.get().class());
        if compact {
            c.push_str(" cl-live--compact");
        }
        c
    };
    view! {
        <span
            class=class
            role="status"
            title=move || compact.then(label)
            aria-label=move || compact.then(label)
        >
            <span class="cl-live__dot" aria-hidden="true"></span>
            {(!compact).then(|| view! { <span class="cl-live__label">{label}</span> })}
        </span>
    }
}

// ============================================================================
// StatTile + Sparkline + SegmentedBar
// ============================================================================

/// A KPI tile: a mono uppercase label, a big value, a unit, and a line under
/// it (a delta and/or a sub line). A slot at the bottom takes a
/// [`Sparkline`].
///
/// - `color` (optional): a hue token for the value (and the sparkline in the
///   slot). Default: the bright text colour.
/// - `delta` (optional): a change, such as `"+12%"`, in `delta_color`
///   (a hue token; default muted).
/// - `sub` (optional): a short line such as `"last 24h"`.
#[component]
pub fn StatTile(
    #[prop(into)] label: String,
    #[prop(into)] value: Signal<String>,
    #[prop(optional, into)] unit: String,
    #[prop(optional, into)] sub: MaybeProp<String>,
    #[prop(optional, into)] delta: MaybeProp<String>,
    #[prop(optional, into)] delta_color: String,
    #[prop(optional, into)] color: String,
    #[prop(optional)] spark: Option<Children>,
) -> impl IntoView {
    let mut style = String::new();
    if !color.is_empty() {
        style.push_str(&format!("--stat-color:{color};"));
    }
    let delta_style = if delta_color.is_empty() {
        String::new()
    } else {
        format!("color:{};", fg_for(&delta_color))
    };
    let has_unit = !unit.is_empty();
    view! {
        <div class="cl-stat" style=style>
            <div class="cl-stat__label">{label}</div>
            <div class="cl-stat__value">
                <span class="cl-tnum">{move || value.get()}</span>
                {has_unit.then(|| view! { <span class="cl-stat__unit">{unit}</span> })}
            </div>
            {move || {
                let d = delta.get().filter(|s| !s.is_empty());
                let s = sub.get().filter(|s| !s.is_empty());
                (d.is_some() || s.is_some()).then(|| {
                    let delta_style = delta_style.clone();
                    view! {
                        <div class="cl-stat__foot">
                            {d.map(|d| view! { <span class="cl-stat__delta" style=delta_style>{d}</span> })}
                            {s.map(|s| view! { <span class="cl-stat__sub">{s}</span> })}
                        </div>
                    }
                })
            }}
            {spark.map(|s| view! { <div class="cl-stat__spark">{s()}</div> })}
        </div>
    }
}

/// A tiny chart of a series: a line (default) or bars. It draws in
/// `currentColor`, so it takes the colour of its parent, or `color`.
///
/// - `bars`: bars in place of a line.
/// - `fill`: a light fill under the line.
/// - `width` / `height`: the size in px (default 120 × 28). `fluid`: the
///   full width of the parent, `height` high.
/// - `zero_base` (default true): the bottom of the chart is 0 when all values
///   are positive. Set it false to show small changes of a large value.
/// - `label`: the accessible name (for example "Requests, last hour"). With
///   no label the chart is hidden from screen readers.
#[component]
pub fn Sparkline(
    #[prop(into)] values: Signal<Vec<f64>>,
    #[prop(optional)] bars: bool,
    #[prop(optional)] fill: bool,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 28.0)] height: f64,
    #[prop(optional)] fluid: bool,
    #[prop(default = true)] zero_base: bool,
    #[prop(optional, into)] color: String,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    let named = !label.is_empty();
    let mut style = String::new();
    if !color.is_empty() {
        style.push_str(&format!("color:{color};"));
    }
    if fluid {
        style.push_str(&format!("width:100%;height:{height}px;"));
    } else {
        style.push_str(&format!("width:{width}px;height:{height}px;"));
    }
    let body = move || {
        let vals = values.get();
        if bars {
            spark_bars(&vals, width, height, 1.5, zero_base)
                .into_iter()
                .map(|(x, y, w, h)| {
                    view! { <rect x=x y=y width=w height=h rx="1" fill="currentColor" /> }
                })
                .collect_view()
                .into_any()
        } else {
            let pts = spark_points(&vals, width, height, zero_base);
            let line = spark_line_path(&pts);
            let area = fill.then(|| spark_area_path(&pts, height));
            view! {
                {area.map(|d| view! {
                    <path class="cl-spark__area" d=d fill="currentColor" stroke="none" />
                })}
                <path
                    class="cl-spark__line"
                    d=line
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    vector-effect="non-scaling-stroke"
                />
            }
            .into_any()
        }
    };
    view! {
        <svg
            class="cl-spark"
            style=style
            viewBox=format!("0 0 {width} {height}")
            preserveAspectRatio="none"
            role=named.then_some("img")
            aria-label=named.then(|| label.clone())
            aria-hidden=(!named).then_some("true")
            focusable="false"
        >
            {body}
        </svg>
    }
}

/// One part of a [`SegmentedBar`].
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub label: String,
    pub value: f64,
    /// A hue token (`token::OK`).
    pub color: String,
}

impl Segment {
    pub fn new(label: impl Into<String>, value: f64, color: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value,
            color: color.into(),
        }
    }
}

/// A bar in parts that shows proportions: ok / warning / failed, or done /
/// active / to do. Each part has a tooltip "label: value".
///
/// - `legend`: a row under the bar with a dot, the label and the value of
///   each part.
/// - `label`: the accessible name of the bar (the parts are read after it).
/// - `height` (default 8 px).
///
/// For one value from 0 to 100, use [`Meter`](crate::widgets::Meter).
#[component]
pub fn SegmentedBar(
    #[prop(into)] segments: Signal<Vec<Segment>>,
    #[prop(optional)] legend: bool,
    #[prop(optional, into)] label: String,
    #[prop(default = 8)] height: u32,
) -> impl IntoView {
    let label = StoredValue::new(label);
    let aria = move || {
        let parts = segments
            .get()
            .iter()
            .map(|s| format!("{} {}", s.label, fmt_num(s.value)))
            .collect::<Vec<_>>()
            .join(", ");
        label.with_value(|l| {
            if l.is_empty() {
                parts
            } else {
                format!("{l}: {parts}")
            }
        })
    };
    let bar = move || {
        let segs = segments.get();
        let widths = segment_widths(&segs.iter().map(|s| s.value).collect::<Vec<_>>());
        segs.into_iter()
            .zip(widths)
            .filter(|(_, w)| *w > 0.0)
            .map(|(s, w)| {
                let title = format!("{}: {}", s.label, fmt_num(s.value));
                view! {
                    <span
                        class="cl-segbar__part"
                        style=format!("width:{w:.3}%;background:{};", s.color)
                        title=title
                    ></span>
                }
            })
            .collect_view()
    };
    let legend_view = move || {
        legend.then(|| {
            let items = segments
                .get()
                .into_iter()
                .map(|s| {
                    view! {
                        <span class="cl-segbar__key">
                            <span class="cl-segbar__swatch" style=format!("background:{};", s.color)></span>
                            {s.label.clone()}
                            <span class="cl-segbar__count cl-tnum">{fmt_num(s.value)}</span>
                        </span>
                    }
                })
                .collect_view();
            view! { <div class="cl-segbar__legend" aria-hidden="true">{items}</div> }
        })
    };
    view! {
        <div class="cl-segbar">
            <div
                class="cl-segbar__track"
                role="img"
                aria-label=aria
                style=format!("height:{height}px;")
            >
                {bar}
            </div>
            {legend_view}
        </div>
    }
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}

// ============================================================================
// DetailList + KeyValue + SectionLabel
// ============================================================================

/// A list of label / value rows (a `<dl>`). Put [`KeyValue`] rows in it.
///
/// - `mono`: all values in the mono font (ids, hashes, times).
/// - `dividers` (default true): a hairline between rows.
/// - `stacked`: the label over the value, in a grid of columns (for a
///   summary strip). Default: the label on the left, the value on the right
///   of it.
/// - `label_width` (default `"140px"`): the width of the label column.
#[component]
pub fn DetailList(
    #[prop(optional)] mono: bool,
    #[prop(default = true)] dividers: bool,
    #[prop(optional)] stacked: bool,
    #[prop(optional, into)] label_width: String,
    children: Children,
) -> impl IntoView {
    let mut class = String::from("cl-kvlist");
    if mono {
        class.push_str(" cl-kvlist--mono");
    }
    if dividers {
        class.push_str(" cl-kvlist--dividers");
    }
    if stacked {
        class.push_str(" cl-kvlist--stacked");
    }
    let style = if label_width.is_empty() {
        String::new()
    } else {
        format!("--kv-label-w:{label_width};")
    };
    view! { <dl class=class style=style>{children()}</dl> }
}

/// One row of a [`DetailList`]: a label and a value (the children: text, a
/// `Pill`, a `RelativeTime`, a link...). `mono` sets this value in the mono
/// font.
#[component]
pub fn KeyValue(
    #[prop(into)] label: String,
    #[prop(optional)] mono: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="cl-kv">
            <dt class="cl-kv__label">{label}</dt>
            <dd class="cl-kv__value" class:cl-mono=mono>{children()}</dd>
        </div>
    }
}

/// A section heading in the mono uppercase "eyebrow" style, with an optional
/// count and an action on the right (a link, a small button).
///
/// - `level` (default 3): the heading level, 2 to 6.
/// - `divider`: a hairline under the heading.
#[component]
pub fn SectionLabel(
    #[prop(into)] label: String,
    #[prop(optional, into)] count: MaybeProp<usize>,
    #[prop(optional)] action: Option<Children>,
    #[prop(optional)] divider: bool,
    #[prop(default = 3)] level: u8,
) -> impl IntoView {
    let inner = view! {
        {label}
        {move || count.get().map(|n| view! { <span class="cl-section-label__count cl-tnum">{n}</span> })}
    };
    let heading = match level {
        2 => view! { <h2 class="cl-section-label__text">{inner}</h2> }.into_any(),
        4 => view! { <h4 class="cl-section-label__text">{inner}</h4> }.into_any(),
        5 => view! { <h5 class="cl-section-label__text">{inner}</h5> }.into_any(),
        6 => view! { <h6 class="cl-section-label__text">{inner}</h6> }.into_any(),
        _ => view! { <h3 class="cl-section-label__text">{inner}</h3> }.into_any(),
    };
    view! {
        <div class="cl-section-label" class:cl-section-label--divider=divider>
            {heading}
            {action.map(|a| view! { <div class="cl-section-label__action">{a()}</div> })}
        </div>
    }
}

// ============================================================================
// CodeBlock + LogView
// ============================================================================

/// A block of code or text: mono, on the inset surface, scrolls in both
/// directions up to `max_height`, with a copy button.
///
/// - `max_height` (default `"320px"`).
/// - `copy` (default true): a copy button at the top right.
/// - `wrap`: wrap long lines in place of a horizontal scroll.
/// - `label`: the accessible name of the scroll area.
#[component]
pub fn CodeBlock(
    #[prop(into)] code: Signal<String>,
    #[prop(default = "320px".to_string(), into)] max_height: String,
    #[prop(default = true)] copy: bool,
    #[prop(optional)] wrap: bool,
    #[prop(default = "Code".to_string(), into)] label: String,
) -> impl IntoView {
    view! {
        <div class="cl-codeblock" class:cl-codeblock--wrap=wrap>
            {copy.then(|| view! {
                <div class="cl-codeblock__tools">
                    <CopyButton value=code icon=true label="Copy code" />
                </div>
            })}
            <pre
                class="cl-codeblock__pre"
                style=format!("max-height:{max_height};")
                tabindex="0"
                aria-label=label
            >
                <code>{move || code.get()}</code>
            </pre>
        </div>
    }
}

/// One line of a [`LogView`].
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LogLine {
    /// A short time, already formatted (`12:04:53`).
    pub time: Option<String>,
    /// A level or an event type, shown as a pill.
    pub level: Option<String>,
    /// The hue token of the level pill. Default: `status_color(level)`.
    pub level_color: Option<String>,
    pub text: String,
}

impl LogLine {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Sets the time prefix.
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// Sets the level pill; its colour comes from `status_color`.
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.level = Some(level.into());
        self
    }

    /// Sets the level pill with its hue token.
    pub fn level_color(mut self, level: impl Into<String>, color: impl Into<String>) -> Self {
        self.level = Some(level.into());
        self.level_color = Some(color.into());
        self
    }
}

/// A log or an event list: mono lines on the inset surface, each with an
/// optional time and level pill. It scrolls up to `max_height`.
///
/// - `follow` (default true): stay at the newest line while lines come in.
///   When the person scrolls up, it stops; a "Jump to latest" button brings
///   it back.
/// - `copy` (default true): a button that copies all lines as text.
/// - `empty`: the text when there are no lines (default "No lines yet.").
/// - `label`: the accessible name (default "Log").
///
/// The region is `role="log"` with `aria-live="off"`: a screen reader can
/// read it, but it does not say each new line.
#[component]
pub fn LogView(
    #[prop(into)] lines: Signal<Vec<LogLine>>,
    #[prop(default = "320px".to_string(), into)] max_height: String,
    #[prop(default = true)] follow: bool,
    #[prop(default = true)] copy: bool,
    #[prop(default = "No lines yet.".to_string(), into)] empty: String,
    #[prop(default = "Log".to_string(), into)] label: String,
) -> impl IntoView {
    let scroller = NodeRef::<html::Div>::new();
    let following = RwSignal::new(follow);
    let scroll_to_end = move || {
        request_animation_frame(move || {
            if let Some(el) = scroller.get_untracked() {
                el.set_scroll_top(el.scroll_height());
            }
        });
    };
    // New lines: keep the newest in view while following.
    Effect::new(move |_| {
        lines.track();
        if follow && following.get_untracked() {
            scroll_to_end();
        }
    });
    let on_scroll = move |_| {
        if !follow {
            return;
        }
        if let Some(el) = scroller.get_untracked() {
            let gap = el.scroll_height() - el.scroll_top() - el.client_height();
            following.set(gap <= 8);
        }
    };
    let text = Signal::derive(move || log_text(&lines.get()));
    let empty = StoredValue::new(empty);
    let rows = move || {
        let ls = lines.get();
        if ls.is_empty() {
            return view! { <div class="cl-log__empty">{empty.get_value()}</div> }.into_any();
        }
        ls.into_iter()
            .map(|l| {
                let pill = l.level.map(|lv| {
                    let c = l
                        .level_color
                        .unwrap_or_else(|| status_color(&lv).to_string());
                    let style = format!("background:{};color:{};", fill_for(&c), fg_for(&c));
                    view! { <span class="cl-pill cl-log__level" style=style>{lv}</span> }
                });
                view! {
                    <div class="cl-log__line">
                        {l.time.map(|t| view! { <span class="cl-log__time">{t}</span> })}
                        {pill}
                        <span class="cl-log__text">{l.text}</span>
                    </div>
                }
            })
            .collect_view()
            .into_any()
    };
    view! {
        <div class="cl-log">
            {copy.then(|| view! {
                <div class="cl-codeblock__tools">
                    <CopyButton value=text icon=true label="Copy log" />
                </div>
            })}
            <div
                node_ref=scroller
                class="cl-log__scroll"
                style=format!("max-height:{max_height};")
                role="log"
                aria-live="off"
                aria-label=label
                tabindex="0"
                on:scroll=on_scroll
            >
                {rows}
            </div>
            {move || (follow && !following.get()).then(|| view! {
                <button
                    type="button"
                    class="cl-log__jump"
                    on:click=move |_| {
                        following.set(true);
                        scroll_to_end();
                    }
                >
                    <IconChevron size=12 />
                    "Jump to latest"
                </button>
            })}
        </div>
    }
}

// ============================================================================
// FeedList + FeedRow
// ============================================================================

/// A list of feed rows (an activity feed, a run feed). Put [`FeedRow`]s in
/// it. `label` is the accessible name of the list.
#[component]
pub fn FeedList(#[prop(optional, into)] label: String, children: Children) -> impl IntoView {
    let label = (!label.is_empty()).then_some(label);
    view! { <ul class="cl-feed" aria-label=label>{children()}</ul> }
}

/// One event in a [`FeedList`]: time, a dot, the subject, a status pill and
/// a line of text (the children).
///
/// - `at` (ms since the epoch): shown as a [`RelativeTime`]. Or `time`: a
///   time that is already formatted.
/// - `subject`: the thing the event is about (mono). `actor`: who did it.
/// - `status`: a pill; its hue is `status_color` unless you give
///   `status_color`.
/// - `dot`: a hue token for a status dot at the start.
/// - `href`: the row is a link. `on_click`: the row is a button. With
///   neither, the row is plain text.
#[component]
pub fn FeedRow(
    #[prop(optional)] at: Option<f64>,
    #[prop(optional, into)] time: String,
    #[prop(optional, into)] subject: String,
    #[prop(optional, into)] actor: String,
    #[prop(optional, into)] status: String,
    #[prop(optional, into)] status_color: String,
    #[prop(optional, into)] dot: String,
    #[prop(optional, into)] href: String,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let time_view = match at {
        Some(t) => view! { <span class="cl-feed__time"><RelativeTime at=t /></span> }.into_any(),
        None if !time.is_empty() => view! { <span class="cl-feed__time">{time}</span> }.into_any(),
        None => view! { <span class="cl-feed__time"></span> }.into_any(),
    };
    let dot_view = (!dot.is_empty()).then(|| {
        view! { <span class="cl-feed__dot" style=format!("background:{dot};") aria-hidden="true"></span> }
    });
    let subject_view =
        (!subject.is_empty()).then(|| view! { <span class="cl-feed__subject">{subject}</span> });
    let actor_view =
        (!actor.is_empty()).then(|| view! { <span class="cl-feed__actor">{actor}</span> });
    let pill = (!status.is_empty()).then(|| {
        let c = if status_color.is_empty() {
            crate::tokens::status_color(&status).to_string()
        } else {
            status_color
        };
        let style = format!("background:{};color:{};", fill_for(&c), fg_for(&c));
        view! { <span class="cl-pill cl-feed__status" style=style>{status}</span> }
    });
    let text = children.map(|c| view! { <span class="cl-feed__text">{c()}</span> });
    let inner = view! { {time_view} {dot_view} {subject_view} {actor_view} {pill} {text} };
    let row = if !href.is_empty() {
        view! { <a class="cl-feed__row cl-feed__row--link" href=href>{inner}</a> }.into_any()
    } else if let Some(cb) = on_click {
        view! {
            <button type="button" class="cl-feed__row cl-feed__row--link" on:click=move |_| cb.run(())>
                {inner}
            </button>
        }
        .into_any()
    } else {
        view! { <div class="cl-feed__row">{inner}</div> }.into_any()
    };
    view! { <li class="cl-feed__item">{row}</li> }
}

// ============================================================================
// Pagination
// ============================================================================

/// Previous / Next buttons, the range ("41–60 of 212"), and an optional page
/// size select, for a list that the server pages with an offset and a limit.
///
/// - `offset` / `limit`: the page; the buttons and the select change them.
/// - `total`: the number of items in the whole list.
/// - `page_sizes` (optional): the choices of the "Rows" select, for example
///   `vec![20, 50, 100]`. With none, there is no select.
/// - `on_change` (optional): runs with `(offset, limit)` after a change.
///
/// A new page size keeps the first item of the page in view.
#[component]
pub fn Pagination(
    offset: RwSignal<usize>,
    limit: RwSignal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(optional)] page_sizes: Vec<usize>,
    #[prop(default = "Previous".to_string(), into)] prev_label: String,
    #[prop(default = "Next".to_string(), into)] next_label: String,
    #[prop(optional)] on_change: Option<Callback<(usize, usize)>>,
) -> impl IntoView {
    let range = Memo::new(move |_| page_range(offset.get(), limit.get(), total.get()));
    let notify = move || {
        if let Some(cb) = on_change {
            cb.run((offset.get_untracked(), limit.get_untracked()));
        }
    };
    let go = move |to: usize| {
        offset.set(to);
        notify();
    };
    let size_id = crate::components::field_id();
    let sizes = (!page_sizes.is_empty()).then(|| {
        let opts = page_sizes
            .iter()
            .map(|n| view! { <option value=n.to_string()>{n.to_string()}</option> })
            .collect_view();
        let size_id2 = size_id.clone();
        view! {
            <span class="cl-pager__size">
                <label class="cl-pager__size-label" for=size_id>"Rows"</label>
                <select
                    id=size_id2
                    class="cl-input cl-select cl-pager__select"
                    prop:value=move || limit.get().to_string()
                    on:change=move |e| {
                        if let Ok(n) = event_target_value(&e).parse::<usize>() {
                            let new_offset = offset_for_limit(offset.get_untracked(), n);
                            limit.set(n);
                            offset.set(new_offset);
                            notify();
                        }
                    }
                >
                    {opts}
                </select>
            </span>
        }
    });
    view! {
        <nav class="cl-pager" aria-label="Pagination">
            <span class="cl-pager__range cl-tnum" aria-live="polite">
                {move || page_range_label(&range.get())}
            </span>
            <span class="cl-pager__controls">
                {sizes}
                <Button
                    variant="default"
                    size="xs"
                    disabled=Signal::derive(move || !range.get().has_prev)
                    on_click=Callback::new(move |_| go(range.get_untracked().prev_offset))
                >
                    <IconChevron size=14 dir="left" />
                    {prev_label}
                </Button>
                <Button
                    variant="default"
                    size="xs"
                    disabled=Signal::derive(move || !range.get().has_next)
                    on_click=Callback::new(move |_| go(range.get_untracked().next_offset))
                >
                    {next_label}
                    <IconChevron size=14 dir="right" />
                </Button>
            </span>
        </nav>
    }
}

// ============================================================================
// Tests (pure logic)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_time_steps() {
        assert_eq!(format_relative(0.0), "just now");
        assert_eq!(format_relative(4_999.0), "just now");
        assert_eq!(format_relative(5_000.0), "5s ago");
        assert_eq!(format_relative(59_999.0), "59s ago");
        assert_eq!(format_relative(60_000.0), "1m ago");
        assert_eq!(format_relative(3_599_000.0), "59m ago");
        assert_eq!(format_relative(3_600_000.0), "1h ago");
        assert_eq!(format_relative(86_400_000.0), "1d ago");
        assert_eq!(format_relative(29.0 * 86_400_000.0), "29d ago");
        assert_eq!(format_relative(45.0 * 86_400_000.0), "1mo ago");
        assert_eq!(format_relative(400.0 * 86_400_000.0), "1y ago");
    }

    #[test]
    fn relative_time_future_and_bad_input() {
        assert_eq!(format_relative(-180_000.0), "in 3m");
        assert_eq!(format_relative(-2_000.0), "just now");
        assert_eq!(format_relative(f64::NAN), "—");
        assert_eq!(format_relative(f64::INFINITY), "—");
    }

    #[test]
    fn duration_steps() {
        assert_eq!(format_duration(0.0), "0ms");
        assert_eq!(format_duration(850.4), "850ms");
        assert_eq!(format_duration(999.4), "999ms");
        assert_eq!(format_duration(1_000.0), "1.0s");
        assert_eq!(format_duration(4_249.0), "4.2s");
        assert_eq!(format_duration(59_940.0), "59.9s");
        // Would round to "60.0s": shows as minutes.
        assert_eq!(format_duration(59_970.0), "1m 00s");
        assert_eq!(format_duration(185_000.0), "3m 05s");
        assert_eq!(format_duration(3_600_000.0), "1h 00m");
        assert_eq!(format_duration(7_440_000.0), "2h 04m");
        assert_eq!(format_duration(266_400_000.0), "3d 2h");
    }

    #[test]
    fn duration_bad_input() {
        assert_eq!(format_duration(-1.0), "—");
        assert_eq!(format_duration(f64::NAN), "—");
    }

    #[test]
    fn page_range_middle_first_last() {
        let r = page_range(40, 20, 212);
        assert_eq!((r.first, r.last, r.total), (41, 60, 212));
        assert!(r.has_prev && r.has_next);
        assert_eq!((r.prev_offset, r.next_offset), (20, 60));
        assert_eq!(page_range_label(&r), "41–60 of 212");

        let first = page_range(0, 20, 212);
        assert!(!first.has_prev && first.has_next);
        assert_eq!(first.prev_offset, 0);

        let last = page_range(200, 20, 212);
        assert_eq!((last.first, last.last), (201, 212));
        assert!(last.has_prev && !last.has_next);
        assert_eq!(last.next_offset, 200);
    }

    #[test]
    fn page_range_empty_and_clamped() {
        let empty = page_range(0, 20, 0);
        assert_eq!(page_range_label(&empty), "0 of 0");
        assert!(!empty.has_prev && !empty.has_next);
        assert_eq!((empty.first, empty.last), (0, 0));

        // Past the end: the start of the last page.
        let past = page_range(500, 20, 212);
        assert_eq!(past.offset, 200);
        assert_eq!(page_range_label(&past), "201–212 of 212");

        // Exactly one full page.
        let one = page_range(0, 20, 20);
        assert_eq!(page_range_label(&one), "1–20 of 20");
        assert!(!one.has_next);

        // A zero limit counts as one.
        assert_eq!(page_range(0, 0, 3).last, 1);
    }

    #[test]
    fn page_size_change_keeps_the_first_item() {
        assert_eq!(offset_for_limit(40, 50), 0);
        assert_eq!(offset_for_limit(60, 50), 50);
        assert_eq!(offset_for_limit(120, 20), 120);
        assert_eq!(offset_for_limit(7, 0), 7);
    }

    #[test]
    fn sort_toggle() {
        let s = SortState::default();
        assert_eq!(s.aria_sort("name"), "none");
        let s = s.toggle("name", SortDir::Asc);
        assert_eq!(s, SortState::by("name", SortDir::Asc));
        assert_eq!(s.aria_sort("name"), "ascending");
        let s = s.toggle("name", SortDir::Asc);
        assert_eq!(s.aria_sort("name"), "descending");
        let s = s.toggle("name", SortDir::Asc);
        assert_eq!(s.aria_sort("name"), "ascending");
        // A new column starts in its own first direction.
        let s = s.toggle("started", SortDir::Desc);
        assert_eq!(s, SortState::by("started", SortDir::Desc));
        assert_eq!(s.aria_sort("name"), "none");
        assert_eq!(s.dir_of("started"), Some(SortDir::Desc));
    }

    #[test]
    fn sort_order_applies_direction() {
        let mut v = vec![3, 1, 2];
        let s = SortState::by("n", SortDir::Desc);
        v.sort_by(|a, b| s.order(a.cmp(b)));
        assert_eq!(v, vec![3, 2, 1]);
        let s = SortState::by("n", SortDir::Asc);
        v.sort_by(|a, b| s.order(a.cmp(b)));
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn spark_points_scale_and_edges() {
        let p = spark_points(&[0.0, 5.0, 10.0], 100.0, 22.0, true);
        assert_eq!(p, vec![(0.0, 21.0), (50.0, 11.0), (100.0, 1.0)]);
        // Not from zero: the smallest value is the bottom.
        let p = spark_points(&[10.0, 20.0], 100.0, 22.0, false);
        assert_eq!(p, vec![(0.0, 21.0), (100.0, 1.0)]);
        // With zero_base, 10 is half way up.
        let p = spark_points(&[10.0, 20.0], 100.0, 22.0, true);
        assert_eq!(p[0], (0.0, 11.0));
    }

    #[test]
    fn spark_points_flat_single_empty_nan() {
        assert!(spark_points(&[], 100.0, 20.0, true).is_empty());
        let flat = spark_points(&[3.0, 3.0], 100.0, 20.0, false);
        assert_eq!(flat, vec![(0.0, 10.0), (100.0, 10.0)]);
        let one = spark_points(&[7.0], 100.0, 20.0, false);
        assert_eq!(one, vec![(50.0, 10.0)]);
        let nan = spark_points(&[f64::NAN, 1.0, 2.0], 100.0, 22.0, false);
        assert_eq!(nan.len(), 2);
    }

    #[test]
    fn spark_paths() {
        assert_eq!(spark_line_path(&[]), "");
        assert_eq!(spark_line_path(&[(0.0, 1.0), (10.0, 5.0)]), "M0 1L10 5");
        assert_eq!(spark_line_path(&[(50.0, 10.0)]), "M48 10L52 10");
        assert_eq!(
            spark_area_path(&[(0.0, 1.0), (10.0, 5.0)], 20.0),
            "M0 1L10 5L10 20L0 20Z"
        );
        assert_eq!(spark_area_path(&[(0.0, 1.0)], 20.0), "");
    }

    #[test]
    fn spark_bar_geometry() {
        let b = spark_bars(&[0.0, 5.0, 10.0, 0.1], 40.0, 20.0, 2.0, true);
        assert_eq!(b.len(), 4);
        // 10px slots, 8px bars centred.
        assert_eq!(b[2], (21.0, 0.0, 8.0, 20.0));
        assert_eq!(b[1], (11.0, 10.0, 8.0, 10.0));
        // Zero has no height; a small value is at least 1px.
        assert_eq!(b[0].3, 0.0);
        assert_eq!(b[3].3, 1.0);
        assert!(spark_bars(&[], 40.0, 20.0, 2.0, true).is_empty());
        // A flat non-zero series fills the height.
        assert_eq!(spark_bars(&[4.0, 4.0], 20.0, 10.0, 0.0, false)[0].3, 10.0);
    }

    #[test]
    fn segment_widths_sum_to_100() {
        let w = segment_widths(&[12.0, 3.0, 0.0, 5.0]);
        assert_eq!(w, vec![60.0, 15.0, 0.0, 25.0]);
        assert!((w.iter().sum::<f64>() - 100.0).abs() < 1e-9);
        assert_eq!(segment_widths(&[0.0, 0.0]), vec![0.0, 0.0]);
        assert_eq!(
            segment_widths(&[-4.0, f64::NAN, 2.0]),
            vec![0.0, 0.0, 100.0]
        );
        assert!(segment_widths(&[]).is_empty());
    }

    #[test]
    fn log_text_joins_parts() {
        let lines = vec![
            LogLine::new("started").time("12:00:01").level("running"),
            LogLine::new("plain line"),
        ];
        assert_eq!(log_text(&lines), "12:00:01 running started\nplain line");
    }
}
