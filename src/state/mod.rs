/// The global editor state.
use termion::event::Key;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;

pub mod document;
pub mod line;

use crate::terminal::Terminal;
use crate::Document;
use crate::Line;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);

pub enum State {
    Normal,
    Replace,
    Insert,
    Prompt,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
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
    status_message: StatusMessage,
}

impl Editor {

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from(" HELP: Ctrl-s = save | Esc = quit");

        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::new(file_name)
            }
        } else {
            Document::default()
        };

        Self {
            quit: false,
            terminal: Terminal::new().expect("Failed to initialize terminal :("),
            cur_pos: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
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
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_pos(&Position {
                x: self.cur_pos.x.saturating_sub(self.offset.x),
                y: self.cur_pos.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mut filename = "[untitled]".to_string();

        if let Some(name) = &self.document.filename {
            filename = name.clone();
            filename.truncate(20);
        }

        status = format!(" {} - {} lines", filename, self.document.len());
        let line_indicator = format!(
            "{}/{}",
            self.cur_pos.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len - 2));
        }

        status = format!("{}{}  ", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn process_insert_keypress(&mut self) -> Result<(), std::io::Error> {
        let key = Terminal::read_key()?;

        match key {
            Key::Esc => self.quit = true,
            Key::Char(c) => {
                self.document.insert(&self.cur_pos, c);

                self.move_cursor(Key::Right);
            },
            Key::Delete => self.document.delete(&self.cur_pos),
            Key::Backspace => {
                if self.cur_pos.x > 0 || self.cur_pos.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cur_pos);
                }
            }
            Key::Ctrl('s') => self.save(),
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

    fn save(&mut self) {
        if self.document.filename.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.filename = new_name;
        } else if self.document.name == true {
            self.document.filename = self.prompt("Save as: ").unwrap_or(None);
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        if self.document.name {
            result = String::from(self.document.filename.as_ref().unwrap());
        }
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;


            match Terminal::read_key()? {
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                },
                Key::Backspace => {
                    if result.len() > 0 {
                        result.remove(result.len() - 1);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
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
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cur_pos;
        let height = self.document.len();
        let mut width = if let Some(line) = self.document.line(y) {
            line.len() + 1
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(line) = self.document.line(y) {
                        x = line.len();
                    } else {
                        x = 0;
                    }
                }
            },
            Key::Right => {
                if x < width.saturating_sub(1) {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            },
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            },
            Key::Home => x = 1,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(line) = self.document.line(y) {
            line.len()
        } else {
            0
        };

        if x > width + 1 {
            x = width + 1;
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
        println!("{}\r", line);
    }

    fn draw_lines(&self) {
        let height = self.terminal.size().height;

        for term_line in 0..height {
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
