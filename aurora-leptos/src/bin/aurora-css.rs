//! `aurora-css [dir]` — writes the Aurora stylesheet to `dir/aurora.css`
//! (default `style/`). Leptos-free; run it from a build hook (e.g. a trunk
//! `pre_build` hook) so the app can `<link>` a real, render-blocking stylesheet:
//!
//! ```toml
//! # Trunk.toml
//! [[hooks]]
//! stage = "pre_build"
//! command = "cargo"
//! command_arguments = ["run", "-q", "-p", "colliery-io-aurora",
//!                      "--no-default-features", "--features", "bin",
//!                      "--bin", "aurora-css", "--", "style"]
//! ```
//!
//! `-p colliery-io-aurora` works only in this repository's workspace: Cargo
//! does not run a binary of a dependency. A product installs this bin
//! (`cargo install colliery-io-aurora --locked --no-default-features
//! --features bin`) or adds a small helper crate that calls
//! `aurora_leptos::write_css` (see the workspace README).
//!
//! `--hyper` also writes `dir/hyper.css`, the opt-in hyper theme.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let hyper = args.iter().any(|a| a == "--hyper");
    let dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "style".to_string());
    let dir = std::path::Path::new(&dir);
    let path = aurora_leptos::write_css(dir).expect("write aurora.css");
    eprintln!("aurora-css: wrote {}", path.display());
    if hyper {
        let path = aurora_leptos::write_hyper_css(dir).expect("write hyper.css");
        eprintln!("aurora-css: wrote {}", path.display());
    }
}
