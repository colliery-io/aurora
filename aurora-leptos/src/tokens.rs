//! Aurora semantic tokens + defensive classifiers — generic, pure logic with no
//! framework dependency. This is the general core shared by every Colliery
//! project; the higher-level state vocab (graph-health/reactor terms) lives
//! alongside the data-display widgets in the `widgets` module.

/// Semantic palette, as CSS variable references.
///
/// Each constant is a `var(--x)` string, not a hex value. Pass it to an inline
/// `style` (`color`, `background`, `stroke`, ...) and the colour follows the
/// theme (light or dark) with no more work. The values live in
/// `style/tokens.css`.
///
/// Three constants per status hue:
/// - `ICE` — the hue: solid fills (dots, meters, count badges) and text on a
///   page or panel surface.
/// - `ICE_FG` — text on the `ICE_BG` fill (pills, badges).
/// - `ICE_BG` — the tinted fill behind `ICE_FG`.
///
/// Because the values are `var(...)` strings, do not add hex alpha to them
/// (`format!("{}1c", token::ICE)` no longer works). Use [`tint`] or
/// [`fill_for`](crate::tokens::fill_for) instead.
pub mod token {
    // The hues.
    pub const ICE: &str = "var(--ice)";
    pub const TEAL: &str = "var(--teal)";
    pub const VIOLET: &str = "var(--violet)";
    pub const GOLD: &str = "var(--gold)";
    pub const OK: &str = "var(--ok)";
    pub const BAD: &str = "var(--bad)";
    pub const SKIP: &str = "var(--skip)";
    pub const MUTED: &str = "var(--muted)";
    pub const FAINT: &str = "var(--faint)";

    // Text on the status fill.
    pub const ICE_FG: &str = "var(--ice-fg)";
    pub const TEAL_FG: &str = "var(--teal-fg)";
    pub const VIOLET_FG: &str = "var(--violet-fg)";
    pub const GOLD_FG: &str = "var(--gold-fg)";
    pub const OK_FG: &str = "var(--ok-fg)";
    pub const BAD_FG: &str = "var(--bad-fg)";
    pub const SKIP_FG: &str = "var(--skip-fg)";
    pub const MUTED_FG: &str = "var(--muted-fg)";

    // The status fill (a tint of the hue).
    pub const ICE_BG: &str = "var(--ice-bg)";
    pub const TEAL_BG: &str = "var(--teal-bg)";
    pub const VIOLET_BG: &str = "var(--violet-bg)";
    pub const GOLD_BG: &str = "var(--gold-bg)";
    pub const OK_BG: &str = "var(--ok-bg)";
    pub const BAD_BG: &str = "var(--bad-bg)";
    pub const SKIP_BG: &str = "var(--skip-bg)";
    pub const MUTED_BG: &str = "var(--muted-bg)";

    /// Text on a solid hue fill (for example a count badge).
    pub const ON_STATUS: &str = "var(--on-status)";
}

/// The status hues that have a `-fg` / `-bg` pair in `tokens.css`.
const PAIRED_HUES: [&str; 8] = [
    "ice", "teal", "violet", "gold", "ok", "bad", "skip", "muted",
];

/// The hue name in `color` if it is one of the paired tokens (`var(--ok)` →
/// `ok`).
fn paired_hue(color: &str) -> Option<&str> {
    let name = color.trim().strip_prefix("var(--")?.strip_suffix(')')?;
    PAIRED_HUES.contains(&name).then_some(name)
}

/// Execution / task status → color. Case-insensitive, muted fallback
/// (REQ-007 defensive rendering — server strings are not a fixed enum).
pub fn status_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "running" => token::ICE,
        "completed" => token::OK,
        "failed" => token::BAD,
        "scheduled" => token::VIOLET,
        "pending" | "paused" => token::MUTED,
        "cancelled" | "canceled" => token::GOLD,
        "skipped" => token::SKIP,
        _ => token::MUTED,
    }
}

/// `color` at `percent` opacity, as a CSS `color-mix()`. Works with any CSS
/// colour: a token (`var(--ice)`), a hex value, a named colour.
///
/// `tint(token::ICE, 13)` → `color-mix(in srgb, var(--ice) 13%, transparent)`.
pub fn tint(color: &str, percent: u8) -> String {
    format!("color-mix(in srgb, {color} {percent}%, transparent)")
}

/// The text colour to put on the tinted fill of `color`.
///
/// For a paired hue token this is its `-fg` token (`var(--ok)` →
/// `var(--ok-fg)`), which meets WCAG AA on the fill in both themes. Any other
/// colour is returned as it is.
pub fn fg_for(color: &str) -> String {
    match paired_hue(color) {
        Some(name) => format!("var(--{name}-fg)"),
        None => color.to_string(),
    }
}

/// The tinted fill for `color` (pill and badge backgrounds).
///
/// For a paired hue token this is its `-bg` token (`var(--ok)` →
/// `var(--ok-bg)`). Any other colour gets an 11% [`tint`].
pub fn fill_for(color: &str) -> String {
    match paired_hue(color) {
        Some(name) => format!("var(--{name}-bg)"),
        None => tint(color, 11),
    }
}

/// Tinted pill/badge background for `color` (spec §Pills). Same as
/// [`fill_for`]; kept under this name because products call it.
///
/// Before 0.4 this appended `1c` to a hex string. It now works with a CSS
/// variable: `pill_bg(token::ICE)` → `var(--ice-bg)`, and
/// `pill_bg("teal")` → `color-mix(in srgb, teal 11%, transparent)`.
pub fn pill_bg(color: &str) -> String {
    fill_for(color)
}

/// Classified error kind → UI presentation (ported from `errors.ts`).
pub struct Classified {
    pub title: &'static str,
    pub message: String,
    pub code: Option<String>,
    pub retryable: bool,
    /// alert accent color var name (`--bad` or `--gold`)
    pub color_var: &'static str,
}

/// A transport/HTTP error shape for `classify`. Apps map their own error type
/// into this (HTTP status + optional server code/message, or a network failure).
#[derive(Clone, PartialEq)]
pub enum ApiError {
    /// Carries an HTTP status + optional server code/message.
    Http {
        status: u16,
        message: String,
        code: Option<String>,
    },
    /// Transport failure (server unreachable).
    Network,
    Unknown(String),
}

pub fn classify(err: &ApiError) -> Classified {
    match err {
        ApiError::Http {
            status,
            message,
            code,
        } => {
            let (kind_title, color_var, retryable) = match status {
                401 | 403 => ("Not authorized", "--bad", false),
                404 => ("Not found", "--bad", false),
                400 | 422 => ("Invalid request", "--gold", false),
                s if *s >= 500 => ("Something went wrong", "--bad", true),
                _ => ("Something went wrong", "--bad", false),
            };
            Classified {
                title: kind_title,
                message: message.clone(),
                code: code.clone(),
                retryable,
                color_var,
            }
        }
        ApiError::Network => Classified {
            title: "Cannot reach server",
            message: "Could not reach the server. Check the URL and that CORS is enabled.".into(),
            code: None,
            retryable: true,
            color_var: "--bad",
        },
        ApiError::Unknown(m) => Classified {
            title: "Something went wrong",
            message: m.clone(),
            code: None,
            retryable: false,
            color_var: "--bad",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_css_variables() {
        for t in [
            token::ICE,
            token::TEAL,
            token::VIOLET,
            token::GOLD,
            token::OK,
            token::BAD,
            token::SKIP,
            token::MUTED,
            token::FAINT,
            token::OK_FG,
            token::OK_BG,
            token::ON_STATUS,
        ] {
            assert!(t.starts_with("var(--") && t.ends_with(')'), "{t}");
        }
    }

    #[test]
    fn every_paired_token_is_defined_in_the_stylesheet() {
        let css = include_str!("../style/tokens.css");
        for hue in PAIRED_HUES {
            for suffix in ["", "-fg", "-bg"] {
                let decl = format!("--{hue}{suffix}:");
                assert!(css.contains(&decl), "tokens.css has no {decl}");
            }
        }
    }

    #[test]
    fn pill_bg_works_with_a_variable() {
        assert_eq!(pill_bg(token::ICE), "var(--ice-bg)");
        assert_eq!(fg_for(token::ICE), "var(--ice-fg)");
        assert_eq!(
            pill_bg(token::FAINT),
            "color-mix(in srgb, var(--faint) 11%, transparent)"
        );
        assert_eq!(fg_for("red"), "red");
        assert_eq!(fill_for("red"), "color-mix(in srgb, red 11%, transparent)");
    }
}
