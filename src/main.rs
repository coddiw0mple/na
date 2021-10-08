//! Sodium is a next generation Vi-like editor.

#![warn(clippy::all, clippy::pedantic)]
mod state;
mod terminal;

use state::Editor;

pub use state::line::Line;
pub use state::document::Document;

fn main() {
    Editor::default().run();
}