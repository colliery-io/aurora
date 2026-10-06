//! The opt-in hyper theme (AURORA-T-0010): its stylesheet stays out of the
//! default one, sets a dark colour scheme, and drops the glow when the OS asks
//! for more contrast. The contrast of its tokens is in `contrast.rs`.

use aurora_leptos::{AURORA_CSS, HYPER_CSS};

#[test]
fn hyper_is_not_in_the_default_stylesheet() {
    assert!(!AURORA_CSS.contains("data-theme=\"hyper\""));
    assert!(!AURORA_CSS.contains("--aurora-hyper-loaded"));
}

#[test]
fn hyper_has_a_dark_scheme_and_a_marker() {
    let block = &HYPER_CSS[HYPER_CSS
        .find(":root[data-theme=\"hyper\"] {")
        .expect("the hyper block")..];
    assert!(block[..block.find("\n}").unwrap()].contains("color-scheme: dark;"));
    assert!(HYPER_CSS.contains(":root { --aurora-hyper-loaded: 1; }"));
}

#[test]
fn every_glow_is_off_with_more_contrast() {
    let guard = "@media not (prefers-contrast: more) {";
    let at = HYPER_CSS.find(guard).expect("the glow block");
    let (outside, inside) = HYPER_CSS.split_at(at);
    for prop in ["box-shadow", "filter"] {
        // Outside the guard, only the tokens (shadows) may name a shadow.
        let stray = outside.lines().find(|l| l.trim_start().starts_with(prop));
        assert!(
            stray.is_none(),
            "a glow outside the contrast guard: {stray:?}"
        );
        assert!(inside.contains(prop), "{prop} in the glow block");
    }
    for target in [
        ".cl-btn--filled",
        ".cl-navlink--active",
        ".cl-dot",
        ".cl-dag__edge",
        ":focus-visible",
    ] {
        assert!(inside.contains(target), "glow on {target}");
    }
}

#[test]
fn write_hyper_css_writes_the_file() {
    let dir = std::env::temp_dir().join(format!("aurora-hyper-{}", std::process::id()));
    let path = aurora_leptos::write_hyper_css(&dir).unwrap();
    assert_eq!(path.file_name().unwrap(), "hyper.css");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HYPER_CSS);
    let _ = std::fs::remove_dir_all(&dir);
}
