use crate::Line;
use std::fs;
use crate::state::Position;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    lines: Vec<Line>,
    pub filename: Option<String>,
    pub name: bool, // In case our file does not exist, we will still set name = true since we may want to name the file with the entered filename
    pub changed: bool, // Has been changed since we opened it?
}

impl Document {

    // Open file with supplied filename
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let data = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for value in data.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self {
            lines,
            filename: Some(filename.to_string()),
            name: true,
            changed: false,
        })
    }

    // Create new file with supplied filename
    pub fn new(filename: &str) -> Self {
        let lines = Vec::new();

        Self {
            lines,
            filename: Some(filename.to_string()),
            name: true,
            changed: false,
        }
    }

    // Saves to given filename in document struct
    pub fn save(&mut self) -> Result<(), Error> {

        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for line in &self.lines {
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.changed = false;
        }
        Ok(())
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.changed = true;

        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut line = Line::default();
            line.insert(0, c);
            self.lines.push(line);
        } else {
            let line = self.lines.get_mut(at.y).unwrap();
            line.insert(at.x, c);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();

        if at.y >= len {
            return;
        }
        self.changed = true;

        if at.x == self.lines.get(at.y).unwrap().len() && at.y < len - 1 {
            let next_line = self.lines.remove(at.y + 1);
            let line = self.lines.get_mut(at.y).unwrap();
            line.append(&next_line);
        } else {
            let line = self.lines.get_mut(at.y).unwrap();
            line.delete(at.x);
        }
    }

    fn insert_newline(&mut self, at: &Position) {

        if at.y == self.len() {
            self.lines.push(Line::default());
        }

        let new_line = self.lines.get_mut(at.y).unwrap().split(at.x);
        self.lines.insert(at.y + 1, new_line);
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

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}