//! WCAG AA contrast for the Aurora tokens, in the light, the dark and the
//! hyper theme.
//!
//! The test reads `style/tokens.css`, takes the light and the dark value of
//! each `light-dark()` token, and the hyper value from `style/hyper.css`
//! (a token that hyper does not set takes its dark value), and computes the contrast of each text/fill pair
//! that the components use. It fails when a pair is below its AA minimum:
//! 4.5:1 for normal text, 3:1 for large text and for the edges of controls.
//!
//! To see the full table: `cargo test -p colliery-io-aurora --test contrast -- --nocapture`.

use std::collections::HashMap;

const TOKENS_CSS: &str = include_str!("../style/tokens.css");
const HYPER_CSS: &str = include_str!("../style/hyper.css");

#[derive(Clone, Copy, Debug)]
struct Rgba {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Light,
    Dark,
    Hyper,
}

/// A parsed token value, before a theme is chosen.
#[derive(Clone, Debug)]
enum Value {
    /// `light-dark(<light>, <dark>)`.
    Pair(String, String),
    /// `color-mix(in srgb, var(--x) N%, transparent)`.
    Tint(String, f64),
}

fn split_top_level(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    out.push(cur.trim().to_string());
    out
}

/// Every `--name: value;` in the token file whose value is a colour we can
/// check. Tokens that are not colours (radii, fonts, shadows) are skipped.
fn parse_tokens() -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for line in TOKENS_CSS.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("--") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_end_matches(';').trim();
        if let Some(inner) = value
            .strip_prefix("light-dark(")
            .and_then(|v| v.strip_suffix(')'))
        {
            let parts = split_top_level(inner);
            assert_eq!(parts.len(), 2, "--{name}: light-dark() needs two values");
            map.insert(
                name.to_string(),
                Value::Pair(parts[0].clone(), parts[1].clone()),
            );
        } else if let Some(inner) = value
            .strip_prefix("color-mix(in srgb, var(--")
            .and_then(|v| v.strip_suffix("%, transparent)"))
        {
            let (base, pct) = inner.split_once(") ").expect("color-mix shape");
            map.insert(
                name.to_string(),
                Value::Tint(base.to_string(), pct.parse().unwrap()),
            );
        }
    }
    map
}

/// Every `--name: value;` in the `:root[data-theme="hyper"]` block of
/// `hyper.css`, colour or not.
fn parse_hyper() -> HashMap<String, String> {
    let start = HYPER_CSS
        .find(":root[data-theme=\"hyper\"] {")
        .expect("hyper.css has the hyper block");
    let block = &HYPER_CSS[start..];
    let block = &block[..block.find("\n}").expect("hyper block end")];
    let mut map = HashMap::new();
    for line in block.lines() {
        let Some(rest) = line.trim().strip_prefix("--") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        map.insert(
            name.to_string(),
            value.trim().trim_end_matches(';').trim().to_string(),
        );
    }
    map
}

fn is_colour(v: &str) -> bool {
    v.starts_with('#') || v.starts_with("rgba(")
}

fn parse_colour(s: &str) -> Rgba {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let v = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f64;
        return match hex.len() {
            6 => Rgba {
                r: v(0),
                g: v(2),
                b: v(4),
                a: 1.0,
            },
            _ => panic!("use 6-digit hex in tokens.css: #{hex}"),
        };
    }
    if let Some(inner) = s.strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) {
        let p: Vec<f64> = inner
            .split(',')
            .map(|x| x.trim().parse().unwrap())
            .collect();
        return Rgba {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        };
    }
    panic!("unknown colour: {s}");
}

struct Palette {
    tokens: HashMap<String, Value>,
    /// The colour values of `:root[data-theme="hyper"]`.
    hyper: HashMap<String, String>,
    theme: Theme,
}

impl Palette {
    fn get(&self, name: &str) -> Rgba {
        if self.theme == Theme::Hyper {
            if let Some(v) = self.hyper.get(name) {
                return parse_colour(v);
            }
        }
        match self.tokens.get(name) {
            Some(Value::Pair(l, d)) => {
                let v = if self.theme == Theme::Light { l } else { d };
                match v.strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
                    Some(other) => self.get(other),
                    None => parse_colour(v),
                }
            }
            Some(Value::Tint(base, pct)) => {
                let c = self.get(base);
                Rgba {
                    a: pct / 100.0,
                    ..c
                }
            }
            None => panic!("no colour token --{name}"),
        }
    }
}

/// `top` painted over the opaque `bottom`, in sRGB like a browser does.
fn over(top: Rgba, bottom: Rgba) -> Rgba {
    let a = top.a;
    Rgba {
        r: top.r * a + bottom.r * (1.0 - a),
        g: top.g * a + bottom.g * (1.0 - a),
        b: top.b * a + bottom.b * (1.0 - a),
        a: 1.0,
    }
}

fn tint(c: Rgba, pct: f64) -> Rgba {
    Rgba {
        a: pct / 100.0,
        ..c
    }
}

fn luminance(c: Rgba) -> f64 {
    let ch = |v: f64| {
        let v = v / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b)
}

fn ratio(a: Rgba, b: Rgba) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

struct Row {
    theme: &'static str,
    what: String,
    ratio: f64,
    need: f64,
}

const SURFACES: &[&str] = &[
    "bg", "sidebar", "panel", "panel-2", "inset", "field", "control",
];
const HUES: &[&str] = &[
    "ice", "teal", "violet", "gold", "ok", "bad", "skip", "muted",
];
const TEXT: &[&str] = &[
    "fg",
    "fg-bright",
    "fg-2",
    "fg-strong",
    "muted",
    "faint",
    "bad-muted",
    "ice",
    "teal",
    "violet",
    "gold",
    "ok",
    "bad",
    "skip",
];

fn check_theme(theme: Theme, rows: &mut Vec<Row>) {
    let p = Palette {
        tokens: parse_tokens(),
        hyper: parse_hyper()
            .into_iter()
            .filter(|(_, v)| is_colour(v))
            .collect(),
        theme,
    };
    let name = match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::Hyper => "hyper",
    };
    let mut push = |what: String, fg: Rgba, bg: Rgba, need: f64| {
        rows.push(Row {
            theme: name,
            what,
            ratio: ratio(fg, bg),
            need,
        });
    };

    // 1. Text tokens on every surface.
    for t in TEXT {
        for s in SURFACES {
            push(format!("--{t} on --{s}"), p.get(t), p.get(s), 4.5);
        }
    }
    // 2. Status text on its own fill, over every surface a pill sits on.
    for h in HUES {
        for s in ["bg", "panel", "panel-2", "inset"] {
            let fill = over(p.get(&format!("{h}-bg")), p.get(s));
            push(
                format!("--{h}-fg on --{h}-bg over --{s}"),
                p.get(&format!("{h}-fg")),
                fill,
                4.5,
            );
        }
    }
    // 3. Text on solid fills.
    for h in HUES {
        push(
            format!("--on-status on --{h}"),
            p.get("on-status"),
            p.get(h),
            4.5,
        );
    }
    for f in ["ice", "ice-5", "ice-7"] {
        push(
            format!("--on-accent on --{f}"),
            p.get("on-accent"),
            p.get(f),
            4.5,
        );
    }
    // 4. Component-specific pairs.
    push(
        "--tooltip-fg on --tooltip-bg".into(),
        p.get("tooltip-fg"),
        p.get("tooltip-bg"),
        4.5,
    );
    for s in ["bg", "panel"] {
        let light_btn = over(tint(p.get("ice"), 14.0), p.get(s));
        push(
            format!("light button: --ice-fg on 14% ice over --{s}"),
            p.get("ice-fg"),
            light_btn,
            4.5,
        );
        let hover_btn = over(tint(p.get("ice"), 22.0), p.get(s));
        push(
            format!("light button hover: --ice-fg on 22% ice over --{s}"),
            p.get("ice-fg"),
            hover_btn,
            4.5,
        );
        let bad_btn = over(tint(p.get("bad"), 16.0), p.get(s));
        push(
            format!("danger light button: --bad-fg on 16% bad over --{s}"),
            p.get("bad-fg"),
            bad_btn,
            4.5,
        );
    }
    for h in ["bad", "gold", "ice", "ok"] {
        let alert = over(tint(p.get(h), 12.0), p.get("panel"));
        push(
            format!("alert title: --{h} on 12% {h} over --panel"),
            p.get(h),
            alert,
            4.5,
        );
        push(
            format!("alert body: --fg on 12% {h} over --panel"),
            p.get("fg"),
            alert,
            4.5,
        );
        let banner = over(tint(p.get(h), 11.0), p.get("panel"));
        push(
            format!("banner text: --fg-2 on 11% {h} over --panel"),
            p.get("fg-2"),
            banner,
            4.5,
        );
    }
    let menu_hover = over(tint(p.get("ice"), 12.0), p.get("panel"));
    push(
        "menu hover: --fg-bright on 12% ice over --panel".into(),
        p.get("fg-bright"),
        menu_hover,
        4.5,
    );
    // Frame (COLLIERY-T-1832): side nav, toasts, tabs.
    push(
        "active nav link: --fg-bright on --ice-bg over --sidebar".into(),
        p.get("fg-bright"),
        over(p.get("ice-bg"), p.get("sidebar")),
        4.5,
    );
    for h in HUES {
        push(
            format!("nav count: --{h}-fg on --{h}-bg over --sidebar"),
            p.get(&format!("{h}-fg")),
            over(p.get(&format!("{h}-bg")), p.get("sidebar")),
            4.5,
        );
    }
    for t in ["fg", "muted"] {
        push(
            format!("toast: --{t} on --control"),
            p.get(t),
            p.get("control"),
            4.5,
        );
    }
    push(
        "segmented active: --fg-bright on --control".into(),
        p.get("fg-bright"),
        p.get("control"),
        4.5,
    );
    push(
        "default button hover: --fg on --control-hover".into(),
        p.get("fg"),
        p.get("control-hover"),
        4.5,
    );
    // Data components (COLLIERY-T-1834).
    for s in ["panel", "panel-2"] {
        push(
            format!("selected table row: --fg on --ice-bg over --{s}"),
            p.get("fg"),
            over(p.get("ice-bg"), p.get(s)),
            4.5,
        );
    }
    for t in ["fg", "muted"] {
        push(
            format!("feed row hover: --{t} on --control-hover"),
            p.get(t),
            p.get("control-hover"),
            4.5,
        );
    }
    for t in ["fg", "fg-2", "fg-bright", "faint", "muted"] {
        push(
            format!("code / log / secret: --{t} on --inset"),
            p.get(t),
            p.get("inset"),
            4.5,
        );
    }
    for h in ["ok", "gold", "muted"] {
        push(
            format!("live indicator: --{h}-fg on --{h}-bg over --panel"),
            p.get(&format!("{h}-fg")),
            over(p.get(&format!("{h}-bg")), p.get("panel")),
            4.5,
        );
    }
    push(
        "secret warning: --gold-fg on --gold-bg over --panel".into(),
        p.get("gold-fg"),
        over(p.get("gold-bg"), p.get("panel")),
        4.5,
    );
    push(
        "section count: --muted-fg on --muted-bg over --bg".into(),
        p.get("muted-fg"),
        over(p.get("muted-bg"), p.get("bg")),
        4.5,
    );
    push(
        "danger menu item: --bad-fg on --bad-bg over --panel".into(),
        p.get("bad-fg"),
        over(p.get("bad-bg"), p.get("panel")),
        4.5,
    );
    for t in ["muted", "faint", "fg-bright"] {
        push(
            format!("stat tile: --{t} on --panel"),
            p.get(t),
            p.get("panel"),
            4.5,
        );
    }
    for h in HUES {
        push(
            format!("sparkline / segment --{h} vs --panel"),
            p.get(h),
            p.get("panel"),
            3.0,
        );
    }
    // 5. Edges of controls and graphics (3:1).
    for s in ["bg", "sidebar", "panel", "panel-2"] {
        push(
            format!("--border-control vs --{s}"),
            p.get("border-control"),
            p.get(s),
            3.0,
        );
        push(format!("--edge vs --{s}"), p.get("edge"), p.get(s), 3.0);
        push(
            format!("--ice (focus ring, switch on) vs --{s}"),
            p.get("ice"),
            p.get(s),
            3.0,
        );
        push(
            format!("--fainter (large text) on --{s}"),
            p.get("fainter"),
            p.get(s),
            3.0,
        );
    }
    for h in HUES {
        push(
            format!("meter fill --{h} vs track --inset"),
            p.get(h),
            p.get("inset"),
            3.0,
        );
    }
}

#[test]
fn tokens_meet_wcag_aa_in_every_theme() {
    let mut rows = Vec::new();
    check_theme(Theme::Light, &mut rows);
    check_theme(Theme::Dark, &mut rows);
    check_theme(Theme::Hyper, &mut rows);

    println!("| theme | pair | ratio | AA min | result |");
    println!("|---|---|---|---|---|");
    for r in &rows {
        let ok = if r.ratio >= r.need { "pass" } else { "FAIL" };
        println!(
            "| {} | {} | {:.2} | {} | {} |",
            r.theme, r.what, r.ratio, r.need, ok
        );
    }
    let fails: Vec<String> = rows
        .iter()
        .filter(|r| r.ratio < r.need)
        .map(|r| format!("{} {}: {:.2} < {}", r.theme, r.what, r.ratio, r.need))
        .collect();
    assert!(
        fails.is_empty(),
        "contrast below WCAG AA:\n{}",
        fails.join("\n")
    );
}

/// Hyper sets each token that has a light and a dark value, so no light or
/// dark value shows through in hyper.
#[test]
fn hyper_sets_every_themed_token() {
    let hyper = parse_hyper();
    let missing: Vec<&str> = TOKENS_CSS
        .lines()
        .filter_map(|l| l.trim().strip_prefix("--"))
        .filter(|l| l.contains("light-dark("))
        .filter_map(|l| l.split_once(':').map(|(name, _)| name))
        .filter(|name| !hyper.contains_key(*name))
        .collect();
    assert!(missing.is_empty(), "hyper.css does not set: {missing:?}");
}
