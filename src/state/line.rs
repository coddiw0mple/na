use std::cmp;

pub struct Line {
    string: String
}

impl From<&str> for Line {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
        }
    }
}

impl Line {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);

        self.string.get(start..end).unwrap_or_default().to_string()
    }
}