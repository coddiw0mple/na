//! Sodium is a next generation Vi-like editor.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction
)]

#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]
mod state;
mod terminal;

use state::Editor;

pub use state::line::Line;
pub use state::document::Document;

use std::thread;
use std::time::Duration;

fn main() {
    Editor::default().run();
}