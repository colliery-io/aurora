//! # Aurora — Colliery's Leptos design system
//!
//! Published on crates.io as **`colliery-io-aurora`** (org-prefixed to avoid
//! claiming generic names); the library itself is imported as `aurora_leptos`.
//!
//! A general design system for Leptos, with a light and a dark theme, reused
//! across Colliery projects. It
//! is the **core that control-plane apps (cloacina included) are built from** —
//! everything below is first-class core, not an optional add-on:
//!
//! - **Components** ([`components`]) — the full Mantine-primitive + Aurora surface:
//!   layout (Box/Group/Stack/SimpleGrid/Grid/AppShell), inputs (Button/TextInput/
//!   Textarea/PasswordInput/NumberInput/Select/Switch/SegmentedControl/CopyButton/
//!   ActionIcon), data + overlay (Table/Tooltip/Modal/Menu/Alert/Loader), and the
//!   Aurora pieces (Pill/StatusBadge/Dot/Panel/PageHeader/Chip/Loading/Empty/
//!   ErrorState).
//! - **Frame** ([`frame`], re-exported from [`components`]) — the page chrome
//!   and the layers over a page: `AppShell` + `SideNav`, `PageHeader`, `Modal`,
//!   `Drawer`, `ConfirmDialog`, `ToastStack` + `Toaster`, `Tabs`, `Card`.
//! - **Widgets** ([`widgets`]) — generic data-display building blocks: `Meter`,
//!   `Banner`, `StateCounts`, `HealthPill`, `BuildStatusBadge`, `NodeReadiness`,
//!   `InputTable`, `StaleInputsBanner`, plus the `Input` model and
//!   `format_ago`/`is_stale`/`freshness_pct` helpers. Apps supply their own state
//!   labels/colors as data — no built-in vocab or branding.
//! - **Graph** ([`graph`], [`graph_layout`]) — the Aurora graph. The product
//!   gives nodes, edges and lanes (`DagNode`, `DagEdge`, `DagLane`); Aurora
//!   does the layout (a pure, tested function) and the interaction (select,
//!   open, hover, keyboard) in the `Dag` component, with `DagLegend`. The old
//!   `Graph` component stays as a thin wrapper.
//! - **Tokens + pure logic** ([`tokens`]) — semantic palette as CSS variables
//!   (`token::ICE` is `"var(--ice)"`), text/fill pairs, `status_color`,
//!   `pill_bg`/`fill_for`/`fg_for`/`tint`, and `ApiError` error classification.
//!   Framework-agnostic Rust.
//! - **Theme** ([`theme`]) — light, dark or system: [`ThemeToggle`],
//!   `set_theme`, `current_theme`, `use_theme`, and [`THEME_INIT_SCRIPT`] for a
//!   first paint with no flash.
//!
//! Genuinely app-specific surfaces (side panels, node detail views, branding)
//! are built downstream from these primitives. The graph layout and its
//! interaction are Aurora's; a product gives only the data.
//!
//! ## Stylesheet
//! The CSS ships inside this crate. Two ways to load it (see the workspace README
//! for full snippets):
//! - **Runtime:** render [`AuroraStyles`] once at the app root (or inject the
//!   [`AURORA_CSS`] const). Simplest; possible first-paint flash.
//! - **Linked stylesheet (no flash):** materialise it as a file and `<link>` it.
//!   With `cargo-leptos` (builds before bundling), call [`write_css`] from
//!   `build.rs`. With `trunk` (validates assets before building), generate it in a
//!   `pre_build` hook via the leptos-free `aurora-css` bin.
//!
//! ```ignore
//! use aurora_leptos::{components::*, tokens::token};
//! view! { <Button>"Run"</Button> <Pill color=token::ICE>"tag"</Pill> }
//! ```

// Pure logic (no renderer) — always available.
// The graph layout: pure, no Leptos (always available).
pub mod graph_layout;
pub mod theme;
pub mod tokens;
pub use theme::*;
pub use tokens::*;

// UI surface — requires the `components` feature (the default).
#[cfg(feature = "components")]
pub mod components;
#[cfg(feature = "components")]
pub mod frame;
#[cfg(feature = "components")]
pub mod graph;
#[cfg(feature = "components")]
pub mod widgets;
#[cfg(feature = "components")]
pub use components::*;
#[cfg(feature = "components")]
pub use graph::*;
#[cfg(feature = "components")]
pub use widgets::*;

// ---- Stylesheet (available with or without the `components` feature) ----

/// The full Aurora stylesheet (IBM Plex `@font-face` + tokens + component
/// chrome + graph chrome), concatenated at compile time.
pub const AURORA_CSS: &str = concat!(
    include_str!("../style/fonts.css"),
    "\n",
    include_str!("../style/tokens.css"),
    "\n",
    include_str!("../style/components.css"),
    "\n",
    include_str!("../style/graph.css"),
);

/// Just the design tokens (CSS custom properties + scales).
pub const TOKENS_CSS: &str = include_str!("../style/tokens.css");
/// Just the component chrome (depends on the token custom properties).
pub const COMPONENTS_CSS: &str = include_str!("../style/components.css");
/// Just the graph chrome (`Dag`, `DagLegend`, `Graph`; depends on the tokens).
pub const GRAPH_CSS: &str = include_str!("../style/graph.css");
/// Just the IBM Plex `@font-face` declarations.
pub const FONTS_CSS: &str = include_str!("../style/fonts.css");

/// Writes the full stylesheet to `dir/aurora.css` and returns the path. Leptos-free
/// — call it from `build.rs` (cargo-leptos) or via the `aurora-css` bin in a trunk
/// `pre_build` hook to ship Aurora as a normal, render-blocking stylesheet.
/// Use `default-features = false` so leptos isn't built for the host.
pub fn write_css(dir: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("aurora.css");
    std::fs::write(&path, AURORA_CSS)?;
    Ok(path)
}

/// Injects the complete stylesheet as an inline `<style>` (runtime fallback for
/// CSR-only setups). Prefer a build-time `<link>` (see [`write_css`]) to avoid a
/// first-paint flash.
#[cfg(feature = "components")]
mod styles {
    use super::AURORA_CSS;
    use leptos::prelude::*;

    #[component]
    pub fn AuroraStyles() -> impl IntoView {
        view! { <style>{AURORA_CSS}</style> }
    }
}
#[cfg(feature = "components")]
pub use styles::AuroraStyles;

// Aurora as a Hlin design pack. Additive and off by default: with the feature
// off, nothing here is compiled and this crate does not know Hlin exists.
#[cfg(feature = "hlin")]
pub mod hlin;
#[cfg(feature = "hlin")]
pub use hlin::{AuroraPack, HLIN_CSS};
