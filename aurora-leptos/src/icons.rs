//! A small icon set: SVG components that draw with `currentColor`.
//!
//! Each icon is a component with two props:
//! - `size` (default 16): the width and height in px.
//! - `title` (optional): the accessible name. With a title the icon is
//!   `role="img"`; without one it is `aria-hidden` (the text next to it
//!   says what it means).
//!
//! The icon takes the text colour of its parent, so it follows the theme and
//! the state of the control it is in (hover, disabled, danger).
//!
//! ```ignore
//! use aurora_leptos::icons::*;
//! view! { <button class="cl-action-icon" aria-label="Run"><IconPlay /></button> }
//! ```

use leptos::prelude::*;

/// One shape of an icon, in a 24 × 24 box.
#[derive(Clone, Copy)]
enum Shape {
    Path(&'static str),
    Circle(f32, f32, f32),
    /// x, y, width, height, corner radius.
    Rect(f32, f32, f32, f32, f32),
}

fn shape_view(s: Shape) -> AnyView {
    match s {
        Shape::Path(d) => view! { <path d=d /> }.into_any(),
        Shape::Circle(cx, cy, r) => view! { <circle cx=cx cy=cy r=r /> }.into_any(),
        Shape::Rect(x, y, w, h, rx) => view! { <rect x=x y=y width=w height=h rx=rx /> }.into_any(),
    }
}

/// Draws `shapes` stroked (outline) or filled with `currentColor`.
fn icon(
    shapes: &'static [Shape],
    filled: bool,
    size: u32,
    title: String,
    class: &'static str,
) -> impl IntoView {
    let named = !title.is_empty();
    let (fill, stroke) = if filled {
        ("currentColor", "none")
    } else {
        ("none", "currentColor")
    };
    view! {
        <svg
            class=format!("cl-icon {class}")
            width=size
            height=size
            viewBox="0 0 24 24"
            fill=fill
            stroke=stroke
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            role=named.then_some("img")
            aria-label=named.then(|| title.clone())
            aria-hidden=(!named).then_some("true")
            focusable="false"
        >
            {shapes.iter().copied().map(shape_view).collect_view()}
        </svg>
    }
}

macro_rules! icons {
    ($( $(#[$doc:meta])* $name:ident, $filled:expr, [$($shape:expr),* $(,)?]; )*) => {
        $(
            $(#[$doc])*
            #[component]
            pub fn $name(
                #[prop(default = 16)] size: u32,
                #[prop(optional, into)] title: String,
            ) -> impl IntoView {
                const SHAPES: &[Shape] = &[$($shape),*];
                icon(SHAPES, $filled, size, title, "")
            }
        )*
    };
}

use Shape::{Circle, Path, Rect};

icons! {
    /// A right-pointing triangle (run, start).
    IconPlay, true, [Path("M7 4v16l13-8z")];
    /// Two bars (pause).
    IconPause, true, [Rect(6.0, 4.0, 4.0, 16.0, 1.0), Rect(14.0, 4.0, 4.0, 16.0, 1.0)];
    /// A lightning bolt (fire, trigger).
    IconBolt, false, [Path("M13 3l-9 13h8l-1 5 9-13h-8l1-5z")];
    /// Two sheets (copy to the clipboard).
    IconCopy, false, [
        Rect(8.0, 8.0, 12.0, 12.0, 2.0),
        Path("M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"),
    ];
    /// An × (close, remove).
    IconClose, false, [Path("M18 6L6 18M6 6l12 12")];
    /// A box with an arrow out of it (opens somewhere else).
    IconExternal, false, [
        Path("M12 6H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6"),
        Path("M11 13l9-9"),
        Path("M15 4h5v5"),
    ];
    /// A tick (done, copied).
    IconCheck, false, [Path("M5 12l5 5L20 7")];
    /// A triangle with "!" (warning).
    IconAlert, false, [
        Path("M12 9v4"),
        Path("M10.36 3.59L2.26 17.13a1.91 1.91 0 0 0 1.64 2.87h16.2a1.91 1.91 0 0 0 1.64-2.87L13.64 3.59a1.91 1.91 0 0 0-3.28 0z"),
        Path("M12 16h.01"),
    ];
    /// A circle with "i" (information).
    IconInfo, false, [Circle(12.0, 12.0, 9.0), Path("M12 8h.01"), Path("M11 12h1v4h1")];
    /// Three lines (open a menu).
    IconMenu, false, [Path("M4 6h16M4 12h16M4 18h16")];
    /// A magnifier (search).
    IconSearch, false, [Circle(10.0, 10.0, 7.0), Path("M21 21l-6-6")];
    /// A sun (light theme).
    IconSun, false, [
        Circle(12.0, 12.0, 4.0),
        Path("M3 12h1m8-9v1m8 8h1m-9 8v1M5.6 5.6l.7.7m12.1-.7l-.7.7m0 11.4l.7.7m-12.1-.7l-.7.7"),
    ];
    /// A moon (dark theme).
    IconMoon, false, [Path("M12 3a7.5 7.5 0 0 0 8.7 12.1A9 9 0 1 1 12 3z")];
    /// A screen (follow the system setting).
    IconMonitor, false, [
        Rect(3.0, 4.0, 18.0, 12.0, 1.0),
        Path("M7 20h10"),
        Path("M9 16v4"),
        Path("M15 16v4"),
    ];
}

/// A chevron. `dir` is `"down"` (default), `"up"`, `"left"` or `"right"`.
#[component]
pub fn IconChevron(
    #[prop(default = 16)] size: u32,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] dir: String,
) -> impl IntoView {
    const SHAPES: &[Shape] = &[Path("M6 9l6 6 6-6")];
    let class = match dir.as_str() {
        "up" => "cl-icon--up",
        "left" => "cl-icon--left",
        "right" => "cl-icon--right",
        _ => "",
    };
    icon(SHAPES, false, size, title, class)
}
