//! AURORA-T-0014: a callback that runs after a component is gone (a frame
//! callback, a timer) does not read the component's nodes or values with a
//! read that panics when they are disposed.
//!
//! A browser frame does not run in a native test (the request for one is a
//! browser call), so each frame callback in the source is checked, and Dag is
//! mounted and dropped. The gallery's "Add a line and rebuild" button is the
//! test of LogView in a browser: before the repair each click panicked.

use aurora_leptos::graph::{Dag, DagEdge, DagNode};
use leptos::prelude::*;

const SOURCES: [(&str, &str); 4] = [
    ("components.rs", include_str!("../src/components.rs")),
    ("data.rs", include_str!("../src/data.rs")),
    ("frame.rs", include_str!("../src/frame.rs")),
    ("graph.rs", include_str!("../src/graph.rs")),
];

/// The body of each `request_animation_frame(move || { … })` in a source.
fn frame_callbacks(src: &str) -> Vec<&str> {
    src.match_indices("request_animation_frame(move ||")
        .map(|(at, _)| {
            let body = &src[at..];
            let mut depth = 0_i32;
            for (i, c) in body.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return &body[..=i];
                        }
                    }
                    _ => {}
                }
            }
            body
        })
        .collect()
}

#[test]
fn frame_callbacks_read_with_try() {
    let mut seen = 0;
    for (name, src) in SOURCES {
        for body in frame_callbacks(src) {
            seen += 1;
            // `.get_untracked()` panics on a disposed node or signal;
            // `.try_get_untracked()` gives None.
            let plain = body.replace(".try_get_untracked()", "");
            assert!(!plain.contains(".get_untracked()"), "{name}: {body}");
            assert!(!plain.contains(".get()"), "{name}: {body}");
        }
    }
    // LogView, Menu, AppShell.
    assert!(seen >= 3, "found {seen} frame callbacks");
}

#[test]
fn graph_cancels_a_pending_select_when_it_goes() {
    let src = include_str!("../src/graph.rs");
    assert!(src.contains("on_cleanup(cancel_pending);"));
}

/// Mounts a view under an owner, runs `then`, and drops the owner.
fn mount_and_drop(view: impl FnOnce() -> AnyView, then: impl FnOnce()) {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    let html = owner.with(|| view().to_html());
    assert!(!html.is_empty());
    owner.with(then);
    owner.cleanup();
    drop(owner);
}

#[test]
fn dag_goes_with_a_select_pending() {
    mount_and_drop(
        || {
            view! {
                <Dag
                    nodes=vec![DagNode::new("a", "a"), DagNode::new("b", "b")]
                    edges=vec![DagEdge::new("a", "b")]
                />
            }
            .into_any()
        },
        || {},
    );
}
