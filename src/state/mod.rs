/// The global editor state.
use termion::event::Key;
use std::env;

pub mod document;
pub mod line;

use crate::terminal::Terminal;
use crate::Document;
use crate::Line;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum State {
    Normal,
    Replace,
    Insert,
    Prompt,
}

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    quit: bool,
    terminal: Terminal,
    cur_pos: Position,
    document: Document,
    offset: Position,
}

impl Editor {

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(&file_name).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            quit: false,
            terminal: Terminal::new().expect("Failed to initialize terminal :("),
            cur_pos: Position::default(),
            document,
            offset: Position::default(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if self.quit {
                break;
            }
            if let Err(error) = self.process_insert_keypress() {
                die(&error);
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_pos(&Position::default());
        if self.quit {
            Terminal::clear_screen();
            println!("Goodbye!\r");
        } else {
            self.draw_lines();
            Terminal::cursor_pos(&Position {
                x: self.cur_pos.x.saturating_sub(self.offset.x),
                y: self.cur_pos.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn process_insert_keypress(&mut self) -> Result<(), std::io::Error> {
        let key = Terminal::read_key()?;

        match key {
            Key::Esc => self.quit = true,
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home=> self.move_cursor(key),
            _ => (),
        };
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        let Position{ x, y} = self.cur_pos;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }

    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cur_pos;
        let size = self.terminal.size();
        let height = self.document.len();
        let width = size.width.saturating_sub(1) as usize;

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => x = x.saturating_sub(1),
            Key::Right => {
                if x < width {
                    x = x.saturating_add(1);
                }
            }
            Key::PageUp => y = 0,
            Key::PageDown => y = height,
            Key::Home => x = 1,
            Key::End => x = width,
            _ => (),
        }
        self.cur_pos = Position {x, y};
    }

    fn process_welcome(&self) {

        let mut blank = format!("");

        let mut message1 = format!("Sodium - A next generation Vi-like editor");
        let mut message2 = format!("version {}", VERSION);
        let mut message3 = format!("By Divith et al.");
        let mut message4 = format!("Sodium is FOSS :)");

        self.draw_welcome(&mut message1);
        self.draw_welcome(&mut blank);
        self.draw_welcome(&mut message2);
        self.draw_welcome(&mut message3);
        self.draw_welcome(&mut message4);
    }

    fn draw_welcome(&self, welcome_message: &mut String) {

        Terminal::clear_current_line();
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        let welcome_message = &mut format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_line(&self, line: &Line) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;

        let line = line.render(start, end);
        println!(" {}\r", line);
    }

    fn draw_lines(&self) {
        let height = self.terminal.size().height;

        for term_line in 0..height - 1 {
            Terminal::clear_current_line();

            if let Some(line) = self.document.line(term_line as usize + self.offset.y) {
                self.draw_line(line);
            } else if self.document.is_empty() && term_line == height/3 {
                self.process_welcome();
            } else {
                println!("~\r");
            }
        }
    }
}

fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}