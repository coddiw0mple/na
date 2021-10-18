use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

pub struct Line {
    string: String,
    len: usize,
}

impl From<&str> for Line {
    fn from(slice: &str) -> Self {
        let mut line = Self {
            string: String::from(slice),
            len: 0,
        };
        line.update_len();
        line
    }
}

impl Line {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);

        let mut result = String::new();

        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push_str("    ");
            } else {
                result.push_str(grapheme);
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
}