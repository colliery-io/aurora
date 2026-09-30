//! No raw colour outside the token definitions.
//!
//! Aurora has a light and a dark theme. A colour written as a literal (a hex
//! value, `rgb()`, `rgba()`, `hsl()`, `hsla()`) shows the same in both themes,
//! so it is almost always a bug. Use a token from `style/tokens.css` instead
//! (`var(--x)` in CSS, `token::X` in Rust).
//!
//! The test scans the CSS and the Rust of the crate and of the gallery. It
//! skips `style/tokens.css` (the token definitions) and the lines in
//! [`ALLOW`]. Each entry in `ALLOW` must say why.

use std::fs;
use std::path::{Path, PathBuf};

/// Files where raw colours are the point.
const SKIP_FILES: &[&str] = &[
    // The token definitions: the one place that holds the raw values.
    "style/tokens.css",
    // This test: it names the patterns it looks for.
    "tests/no_raw_colours.rs",
    // The contrast test parses the token values; it holds no colour itself,
    // but its doc comments name the functions.
    "tests/contrast.rs",
];

/// `(file, text on the line)` pairs that may hold a raw colour, with a reason.
const ALLOW: &[(&str, &str, &str)] = &[
    // (file, text, reason) — empty today. Add an entry only with a reason.
];

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("css" | "rs")
        ) {
            out.push(path);
        }
    }
}

/// The raw colours on one line: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`,
/// `rgb(`, `rgba(`, `hsl(`, `hsla(`.
fn raw_colours(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'#' {
            continue;
        }
        let hex = bytes[i + 1..]
            .iter()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        let next = bytes.get(i + 1 + hex).copied();
        // A word boundary after the digits: `#fff;` is a colour, `#fffx` is not.
        let boundary = next.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == b'_' || c == b'-'));
        if matches!(hex, 3 | 4 | 6 | 8) && boundary {
            found.push(line[i..i + 1 + hex].to_string());
        }
    }
    let lower = line.to_ascii_lowercase();
    for f in ["rgb(", "rgba(", "hsl(", "hsla("] {
        let mut from = 0;
        while let Some(at) = lower[from..].find(f) {
            let at = from + at;
            // Not the tail of a longer name.
            let before = lower[..at].chars().last();
            if before.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-')) {
                found.push(f.to_string());
            }
            from = at + f.len();
        }
    }
    found
}

#[test]
fn no_raw_colour_outside_the_tokens() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gallery_dir = crate_dir.join("../leptos-gallery");
    let mut all = Vec::new();
    for dir in ["style", "src", "tests"] {
        files(&crate_dir.join(dir), &mut all);
    }
    let mut gallery = Vec::new();
    files(&gallery_dir.join("src"), &mut gallery);
    let gallery_css = gallery_dir.join("gallery.css");
    if gallery_css.exists() {
        gallery.push(gallery_css);
    }
    all.extend(gallery);
    assert!(all.len() > 5, "found too few files to scan: {all:?}");

    let mut problems = Vec::new();
    for path in &all {
        let rel = path
            .strip_prefix(crate_dir)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());
        if SKIP_FILES.contains(&rel.as_str()) {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let hits = raw_colours(line);
            if hits.is_empty() {
                continue;
            }
            let allowed = ALLOW
                .iter()
                .any(|(f, t, _)| rel.ends_with(f) && line.contains(t));
            if !allowed {
                problems.push(format!(
                    "{rel}:{}: {} — {}",
                    n + 1,
                    hits.join(", "),
                    line.trim()
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "raw colours outside style/tokens.css (use a token):\n{}",
        problems.join("\n")
    );
}

#[test]
fn the_scanner_finds_what_it_should() {
    assert_eq!(raw_colours("color: #fff;"), vec!["#fff"]);
    assert_eq!(raw_colours("background:#7fb2ff1c"), vec!["#7fb2ff1c"]);
    assert_eq!(
        raw_colours("box-shadow: 0 0 4px rgba(0,0,0,.5)"),
        vec!["rgba("]
    );
    assert_eq!(raw_colours("color: HSL(0 0% 0%)"), vec!["hsl("]);
    // Not colours.
    assert!(raw_colours("#[derive(Clone)]").is_empty());
    assert!(raw_colours("href=\"#main\"").is_empty());
    assert!(raw_colours("format!(\"#{id}\")").is_empty());
    assert!(raw_colours("#cafe-latte").is_empty());
    assert!(raw_colours("--rgb(").is_empty());
}
