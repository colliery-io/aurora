//! The gallery puts the no-flash script in its `<head>`, before the stylesheet,
//! as the docs of `THEME_INIT_SCRIPT` tell a product to do.

use aurora_leptos::THEME_INIT_SCRIPT;

#[test]
fn the_gallery_puts_the_init_script_in_its_head() {
    let html = include_str!("../../leptos-gallery/index.html");
    let head = &html[..html
        .find("</head>")
        .expect("gallery index.html has a </head>")];
    let script = head
        .find(THEME_INIT_SCRIPT)
        .expect("gallery head has THEME_INIT_SCRIPT, exactly");
    let stylesheet = head.find("aurora.css").expect("gallery links aurora.css");
    assert!(
        script < stylesheet,
        "the init script must come before the stylesheet"
    );
}

#[test]
fn the_readme_shows_the_same_script() {
    let readme = include_str!("../../README.md");
    assert!(
        readme.contains(THEME_INIT_SCRIPT),
        "README.md must show THEME_INIT_SCRIPT exactly"
    );
}
