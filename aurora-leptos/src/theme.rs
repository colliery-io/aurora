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
//!
//! The pieces a product needs:
//!
//! - [`THEME_INIT_SCRIPT`] — put it in an inline `<script>` in the `<head>` of
//!   `index.html`. It reads the stored choice and sets `data-theme` before the
//!   first paint, so the page does not flash the wrong theme while the WASM
//!   loads.
//! - [`ThemeToggle`] — a Light / Dark / System control for a top bar.
//! - [`set_theme`] and [`current_theme`] — the same thing without a component.
//! - [`provide_theme`] / [`use_theme`] — a [`ThemeContext`] with signals for the
//!   choice and for the theme in effect (for code that must know if the page is
//!   dark, for example a canvas).
//!
//! The choice is stored in `localStorage` under [`THEME_STORAGE_KEY`]. If
//! storage is not available, or throws, the page follows the system and no
//! error shows.

/// The `localStorage` key that holds the choice: `"light"` or `"dark"`. No key
/// means "follow the system".
pub const THEME_STORAGE_KEY: &str = "aurora-theme";

/// A script to run in the `<head>`, before the stylesheet and the WASM.
///
/// It copies a stored `"light"` or `"dark"` choice to `data-theme` (and to the
/// inline `color-scheme`) on `<html>`, so the first paint has the right theme.
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
pub const THEME_INIT_SCRIPT: &str = r#"(function(){try{var t=localStorage.getItem("aurora-theme");if(t==="light"||t==="dark"){var d=document.documentElement;d.setAttribute("data-theme",t);d.style.colorScheme=t;}}catch(e){}})();"#;

/// A theme choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Theme {
    Light,
    Dark,
    /// Follow the operating system (`prefers-color-scheme`).
    #[default]
    System,
}

impl Theme {
    /// The value for `data-theme` and for storage: `"light"`, `"dark"` or
    /// `"system"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    /// Reads a stored value. Unknown text gives `None`.
    pub fn parse(value: &str) -> Option<Theme> {
        match value.trim() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            "system" => Some(Theme::System),
            _ => None,
        }
    }

    /// A short label for a control: "Light", "Dark", "System".
    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
    }
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

    /// The choice now in effect on `<html>`: `Light` or `Dark` if `data-theme`
    /// is set, else `System`. With [`THEME_INIT_SCRIPT`](super::THEME_INIT_SCRIPT)
    /// in the head, this is the stored choice.
    pub fn current_theme() -> Theme {
        root()
            .and_then(|el| el.get_attribute("data-theme"))
            .and_then(|v| Theme::parse(&v))
            .unwrap_or(Theme::System)
    }

    /// The choice in storage, or `System` if there is none or storage fails.
    pub fn stored_theme() -> Theme {
        storage()
            .and_then(|s| s.get_item(THEME_STORAGE_KEY).ok().flatten())
            .and_then(|v| Theme::parse(&v))
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

        /// True if the page shows the dark theme now (reactive). For `System`
        /// this follows the operating system.
        pub fn is_dark(&self) -> bool {
            match self.choice.get() {
                Theme::Light => false,
                Theme::Dark => true,
                Theme::System => self.system_dark.get(),
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
    #[component]
    pub fn ThemeToggle() -> impl IntoView {
        let ctx = use_theme();
        let buttons = [Theme::Light, Theme::Dark, Theme::System]
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
        for t in [Theme::Light, Theme::Dark, Theme::System] {
            assert_eq!(Theme::parse(t.as_str()), Some(t));
        }
        assert_eq!(Theme::parse("sepia"), None);
    }

    #[test]
    fn init_script_uses_the_storage_key_and_catches_errors() {
        assert!(THEME_INIT_SCRIPT.contains(&format!("\"{THEME_STORAGE_KEY}\"")));
        assert!(THEME_INIT_SCRIPT.contains("try{") && THEME_INIT_SCRIPT.contains("catch(e){}"));
        assert!(!THEME_INIT_SCRIPT.contains("</script"));
    }
}
