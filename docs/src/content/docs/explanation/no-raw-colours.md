---
title: Why no raw colours
description: Why Aurora refuses a hex or rgb() colour outside its token files.
sidebar:
  order: 3
---

Aurora has one place for colour values: `style/tokens.css`, and
`style/hyper.css` for hyper. Every other file uses a token. A test,
`tests/no_raw_colours.rs`, fails on a hex, `rgb()` or `hsl()` value anywhere
else in the crate or the gallery.

## A raw colour is a bug in some theme

A token has a value for each theme. A raw colour has one value. If you write
`color: #1f60cc`, the text is right in the light theme and too dark in the
dark theme. The bug shows only in the theme that you did not look at.

Before Aurora had a light theme, the components held colours as hex strings,
and some code added hex alpha to them (`"#7fb2ff" + "1c"`). When the light
theme came, each of these was a bug. The rule stops new ones.

## One place to check

Because each value is in one file, the contrast test can read every value and
check every pair in each theme. A raw colour in a component would be a value
that the test does not see.

## One look across products

Kairos, Cloacina, Brokkr, Weir and Hlin draw with the same tokens. A product
that writes its own hex values drifts from the others. A product that uses
`var(--ok)` for "healthy" shows the same green as each other product, in each
theme.

## The exceptions

The test has an `ALLOW` list for a line that must hold a raw colour. Each
entry must give a reason. The list is empty.
