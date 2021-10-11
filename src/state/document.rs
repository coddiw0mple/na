use crate::Line;
use std::fs;

#[derive(Default)]
pub struct Document {
    lines: Vec<Line>,
}

impl Document {
    pub fn open(filename: &str ) -> Result<Self, std::io::Error> {
        let data = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for value in data.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self { lines })
    }

    pub fn line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}