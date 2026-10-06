//! Light and dark theme: choose, remember, apply.
//!
//! Aurora has a light and a dark value for each colour token (see
//! `style/tokens.css`). Which one the page shows depends on the `data-theme`
//! attribute of `<html>`:
//!
//! | `<html>` | Theme |
//! |---|---|
//! | no `data-theme` | follows the operating system (`prefers-color-scheme`), live |
//! | `data-theme="light"` | light |
//! | `data-theme="dark"` | dark |
//! | `data-theme="hyper"` | hyper: neon on dark (opt in, needs [`HYPER_CSS`](crate::HYPER_CSS)) |
//!
//! The pieces a product needs:
//!
//! - [`THEME_INIT_SCRIPT`] — put it in an inline `<script>` in the `<head>` of
//!   `index.html`. It reads the stored choice and sets `data-theme` before the
//!   first paint, so the page does not flash the wrong theme while the WASM
//!   loads.
//! - [`ThemeToggle`] — a Light / Dark / System control for a top bar. With
//!   `hyper=true` it has a fourth button, Hyper.
//! - [`set_theme`] and [`current_theme`] — the same thing without a component.
//! - [`provide_theme`] / [`use_theme`] — a [`ThemeContext`] with signals for the
//!   choice and for the theme in effect (for code that must know if the page is
//!   dark, for example a canvas).
//!
//! The choice is stored in `localStorage` under [`THEME_STORAGE_KEY`]. If
//! storage is not available, or throws, the page follows the system and no
//! error shows.
//!
//! ## Hyper (opt in)
//!
//! Hyper is a third theme: neon hues on a dark base. A product that wants it
//! loads [`HYPER_CSS`](crate::HYPER_CSS) after the Aurora stylesheet and passes
//! `hyper=true` to [`ThemeToggle`]. Without the hyper stylesheet a stored
//! `"hyper"` counts as `System`: [`current_theme`] and [`stored_theme`] say
//! `System`, and the page follows the operating system.

/// The `localStorage` key that holds the choice: `"light"`, `"dark"` or
/// `"hyper"`. No key
/// means "follow the system".
pub const THEME_STORAGE_KEY: &str = "aurora-theme";

/// A script to run in the `<head>`, before the stylesheet and the WASM.
///
/// It copies a stored `"light"`, `"dark"` or `"hyper"` choice to `data-theme`
/// on `<html>` (and `"light"` or `"dark"` to the inline `color-scheme`; hyper
/// sets its `color-scheme` in its own stylesheet, so with no hyper stylesheet
/// the page follows the OS), so the first paint has the right theme.
/// It does nothing if there is no choice or if storage throws.
///
/// Put it in `index.html` as it is:
///
/// ```html
/// <head>
///   <meta name="color-scheme" content="light dark" />
///   <script>/* THEME_INIT_SCRIPT here */</script>
///   <link rel="stylesheet" href="aurora.css" />
/// </head>
/// ```
///
/// A test checks that the gallery's `index.html` holds this exact text.
pub const THEME_INIT_SCRIPT: &str = r#"(function(){try{var t=localStorage.getItem("aurora-theme");if(t==="light"||t==="dark"||t==="hyper"){var d=document.documentElement;d.setAttribute("data-theme",t);if(t!=="hyper")d.style.colorScheme=t;}}catch(e){}})();"#;

/// A theme choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Theme {
    Light,
    Dark,
    /// Neon hues on a dark base. Opt in: it needs [`HYPER_CSS`](crate::HYPER_CSS).
    Hyper,
    /// Follow the operating system (`prefers-color-scheme`).
    #[default]
    System,
}

impl Theme {
    /// The value for `data-theme` and for storage: `"light"`, `"dark"`,
    /// `"hyper"` or `"system"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::Hyper => "hyper",
            Theme::System => "system",
        }
    }

    /// Reads a stored value. Unknown text gives `None`.
    pub fn parse(value: &str) -> Option<Theme> {
        match value.trim() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            "hyper" => Some(Theme::Hyper),
            "system" => Some(Theme::System),
            _ => None,
        }
    }

    /// A short label for a control: "Light", "Dark", "Hyper", "System".
    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::Hyper => "Hyper",
            Theme::System => "System",
        }
    }

    /// True for the themes with a dark base: dark and hyper. (`System` is
    /// not known here: ask [`ThemeContext::is_dark`].)
    pub fn is_dark_base(self) -> bool {
        matches!(self, Theme::Dark | Theme::Hyper)
    }
}

/// The buttons of [`ThemeToggle`], in order. Hyper is there only when the
/// product opts in.
pub fn toggle_choices(hyper: bool) -> Vec<Theme> {
    let mut out = vec![Theme::Light, Theme::Dark];
    if hyper {
        out.push(Theme::Hyper);
    }
    out.push(Theme::System);
    out
}

#[cfg(feature = "components")]
pub use dom::*;

#[cfg(feature = "components")]
mod dom {
    use super::{Theme, THEME_STORAGE_KEY};
    use leptos::prelude::*;
    use leptos::wasm_bindgen::{closure::Closure, JsCast};

    const DARK_QUERY: &str = "(prefers-color-scheme: dark)";

    fn root() -> Option<web_sys::HtmlElement> {
        web_sys::window()?
            .document()?
            .document_element()?
            .dyn_into::<web_sys::HtmlElement>()
            .ok()
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    /// True if the hyper stylesheet ([`HYPER_CSS`](crate::HYPER_CSS)) is on
    /// the page. It sets `--aurora-hyper-loaded` on `:root`.
    pub fn hyper_available() -> bool {
        let Some(el) = root() else { return false };
        web_sys::window()
            .and_then(|w| w.get_computed_style(&el).ok().flatten())
            .and_then(|s| s.get_property_value("--aurora-hyper-loaded").ok())
            .is_some_and(|v| v.trim() == "1")
    }

    /// `Hyper` with no hyper stylesheet shows the system theme, so it is
    /// `System`.
    fn shown(theme: Theme) -> Theme {
        if theme == Theme::Hyper && !hyper_available() {
            Theme::System
        } else {
            theme
        }
    }

    /// The choice now in effect on `<html>`: `Light`, `Dark` or `Hyper` if
    /// `data-theme` is set, else `System`. `Hyper` with no hyper stylesheet is
    /// `System`. With [`THEME_INIT_SCRIPT`](super::THEME_INIT_SCRIPT) in the
    /// head, this is the stored choice.
    pub fn current_theme() -> Theme {
        root()
            .and_then(|el| el.get_attribute("data-theme"))
            .and_then(|v| Theme::parse(&v))
            .map(shown)
            .unwrap_or(Theme::System)
    }

    /// The choice in storage, or `System` if there is none or storage fails.
    /// A stored `"hyper"` with no hyper stylesheet is `System`.
    pub fn stored_theme() -> Theme {
        storage()
            .and_then(|s| s.get_item(THEME_STORAGE_KEY).ok().flatten())
            .and_then(|v| Theme::parse(&v))
            .map(shown)
            .unwrap_or(Theme::System)
    }

    /// Shows `theme` on the page (sets or removes `data-theme` on `<html>`)
    /// and does not store it.
    pub fn apply_theme(theme: Theme) {
        let Some(el) = root() else { return };
        let style = el.style();
        match theme {
            Theme::Light | Theme::Dark => {
                let _ = el.set_attribute("data-theme", theme.as_str());
                let _ = style.set_property("color-scheme", theme.as_str());
            }
            // The hyper stylesheet sets `color-scheme: dark`. No inline value:
            // with no hyper stylesheet the page follows the OS.
            Theme::Hyper => {
                let _ = el.set_attribute("data-theme", theme.as_str());
                let _ = style.remove_property("color-scheme");
            }
            Theme::System => {
                let _ = el.remove_attribute("data-theme");
                let _ = style.remove_property("color-scheme");
            }
        }
    }

    /// Shows `theme` on the page and stores it. `System` removes the stored
    /// choice. Storage errors are ignored: the page still changes.
    pub fn set_theme(theme: Theme) {
        apply_theme(theme);
        if let Some(s) = storage() {
            let _ = match theme {
                Theme::System => s.remove_item(THEME_STORAGE_KEY),
                _ => s.set_item(THEME_STORAGE_KEY, theme.as_str()),
            };
        }
    }

    /// True if the operating system asks for a dark theme.
    pub fn system_prefers_dark() -> bool {
        web_sys::window()
            .and_then(|w| w.match_media(DARK_QUERY).ok().flatten())
            .map(|m| m.matches())
            .unwrap_or(false)
    }

    /// The theme state for a Leptos app. Get it with [`use_theme`].
    #[derive(Clone, Copy)]
    pub struct ThemeContext {
        /// The choice: light, dark or system.
        pub choice: RwSignal<Theme>,
        /// True while the operating system asks for dark. Updated live.
        pub system_dark: RwSignal<bool>,
    }

    impl ThemeContext {
        /// Changes the theme, stores it, and updates the signals.
        pub fn set(&self, theme: Theme) {
            set_theme(theme);
            self.choice.set(theme);
        }

        /// The choice (reactive).
        pub fn theme(&self) -> Theme {
            self.choice.get()
        }

        /// True if the page shows a dark base now: dark or hyper (reactive).
        /// For `System` this follows the operating system.
        pub fn is_dark(&self) -> bool {
            match self.choice.get() {
                Theme::System => self.system_dark.get(),
                theme => theme.is_dark_base(),
            }
        }
    }

    /// Makes a [`ThemeContext`] and provides it to the children. Call it once,
    /// near the root of the app. It reads the current choice from the page and
    /// listens for changes of the system setting.
    pub fn provide_theme() -> ThemeContext {
        let ctx = ThemeContext {
            choice: RwSignal::new(current_theme()),
            system_dark: RwSignal::new(system_prefers_dark()),
        };
        if let Some(mql) = web_sys::window().and_then(|w| w.match_media(DARK_QUERY).ok().flatten())
        {
            let system_dark = ctx.system_dark;
            let watched = mql.clone();
            let on_change = Closure::<dyn FnMut()>::new(move || {
                system_dark.set(watched.matches());
            });
            let _ =
                mql.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
            // The listener lives as long as the page. provide_theme runs once.
            on_change.forget();
        }
        provide_context(ctx);
        ctx
    }

    /// The [`ThemeContext`] from [`provide_theme`]. If there is none, it makes
    /// one (for this part of the tree).
    pub fn use_theme() -> ThemeContext {
        use_context::<ThemeContext>().unwrap_or_else(provide_theme)
    }

    /// A Light / Dark / System control. Put it in a top bar.
    ///
    /// The choice changes the page at once and is stored in the browser.
    /// Each button tells assistive technology whether it is pressed.
    ///
    /// `hyper=true` adds a Hyper button (Light / Dark / Hyper / System). Load
    /// [`HYPER_CSS`](crate::HYPER_CSS) too, or Hyper shows the system theme.
    #[component]
    pub fn ThemeToggle(#[prop(optional)] hyper: bool) -> impl IntoView {
        let ctx = use_theme();
        let buttons = super::toggle_choices(hyper)
            .into_iter()
            .map(|theme| {
                view! {
                    <button
                        type="button"
                        class="cl-segmented__item"
                        class:cl-segmented__item--active=move || ctx.choice.get() == theme
                        aria-pressed=move || if ctx.choice.get() == theme { "true" } else { "false" }
                        on:click=move |_| ctx.set(theme)
                    >
                        {theme.label()}
                    </button>
                }
            })
            .collect_view();
        view! {
            <div class="cl-segmented cl-theme-toggle" role="group" aria-label="Theme">
                {buttons}
            </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_round_trips_through_its_text() {
        for t in [Theme::Light, Theme::Dark, Theme::Hyper, Theme::System] {
            assert_eq!(Theme::parse(t.as_str()), Some(t));
        }
        assert_eq!(Theme::parse("sepia"), None);
    }

    #[test]
    fn hyper_is_a_dark_base() {
        assert!(Theme::Hyper.is_dark_base() && Theme::Dark.is_dark_base());
        assert!(!Theme::Light.is_dark_base() && !Theme::System.is_dark_base());
    }

    #[test]
    fn the_toggle_shows_hyper_only_when_asked() {
        use Theme::*;
        assert_eq!(toggle_choices(false), vec![Light, Dark, System]);
        assert_eq!(toggle_choices(true), vec![Light, Dark, Hyper, System]);
    }

    #[test]
    fn init_script_accepts_hyper_with_no_inline_colour_scheme() {
        assert!(THEME_INIT_SCRIPT.contains(r#"t==="hyper""#));
        assert!(THEME_INIT_SCRIPT.contains(r#"if(t!=="hyper")d.style.colorScheme=t;"#));
    }

    #[test]
    fn init_script_uses_the_storage_key_and_catches_errors() {
        assert!(THEME_INIT_SCRIPT.contains(&format!("\"{THEME_STORAGE_KEY}\"")));
        assert!(THEME_INIT_SCRIPT.contains("try{") && THEME_INIT_SCRIPT.contains("catch(e){}"));
        assert!(!THEME_INIT_SCRIPT.contains("</script"));
    }
}
