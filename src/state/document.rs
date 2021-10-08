use crate::Line;

#[derive(Default)]
pub struct Document {
    lines: Vec<Line>,
}

impl Document {
    pub fn open() -> Self {
        let mut lines = Vec::new();
        lines.push(Line::from("Hello, World!"));
        Self { lines }
    }

    pub fn line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }
}