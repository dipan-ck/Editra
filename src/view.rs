use std::io::{self};

use crossterm::event::KeyCode;

use crate::{buffer::Buffer, terminal::Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Offset {
    x: usize,
    y: usize,
}

pub struct CursorLocation {
    x: usize,
    y: usize,
}

pub struct View {
    buffer: Buffer,
    pub need_redraw: bool,
    pub cursor_location: CursorLocation,
    pub offset: Offset,
}

impl View {
    pub fn default() -> Self {
        View {
            buffer: Buffer::default(),
            need_redraw: true,
            cursor_location: CursorLocation { x: 0, y: 0 },
            offset: Offset { x: 0, y: 0 },
        }
    }

    pub fn get_cursor_location(&self) -> (usize, usize) {
        let screen_x = self.cursor_location.x.saturating_sub(self.offset.x);
        let screen_y = self.cursor_location.y.saturating_sub(self.offset.y);
        (screen_x, screen_y)
        // (self.cursor_location.x, self.cursor_location.y)
    }

    pub fn update_cursor_location(&mut self, code: KeyCode) -> Result<(), io::Error> {
        let mut x = self.cursor_location.x;
        let mut y = self.cursor_location.y;
        let height = Terminal::size().1 as usize;
        match code {
            KeyCode::Up => {
                y = y.saturating_sub(1);

                if let Some(line) = self.buffer.lines.get(y) {
                    if line.len() < x {
                        x = line.len();
                    }
                }

                if y < self.offset.y {
                    self.offset.y = y;
                }
            }
            KeyCode::Down => {
                y += 1;

                if let Some(line) = self.buffer.lines.get(y) {
                    if line.len() < x {
                        x = line.len();
                    }
                }

                if y >= self.offset.y + height {
                    self.offset.y += 1;
                }
            }
            KeyCode::Left => x = x.saturating_sub(1),
            KeyCode::Right => {
                if let Some(line) = self.buffer.lines.get(y) {
                    if x < line.len() {
                        x += 1;
                    }
                }
            }

            KeyCode::PageUp => {
                y = y.saturating_sub(height);

                if y < self.offset.y {
                    self.offset.y = y;
                }
            }
            KeyCode::PageDown => {
                y += height;

                if y >= self.offset.y + height {
                    self.offset.y = y - height + 1;
                }
            }
            KeyCode::End => {
                x = self.buffer.lines[y].len();
            }
            KeyCode::Home => x = 0,
            _ => {}
        }

        self.cursor_location = CursorLocation {
            x: x as usize,
            y: y as usize,
        };
        self.need_redraw = true;
        Ok(())
    }

    fn render_buffer(&self) -> Result<(), io::Error> {
        Terminal::move_cursor_to(0, 0)?;
        Terminal::clear_terminal()?;
        let height = Terminal::size().1 as usize;
        let width = Terminal::size().0 as usize;

        for curr_row in 0..height {
            if let Some(line) = self.buffer.lines.get(curr_row + self.offset.y) {
                self.render_line(curr_row as u16, line.get(..width).unwrap_or(line))?;
            } else {
                self.render_line(curr_row as u16, "~")?;
            }
        }

        Ok(())
    }

    fn render_line(&self, row: u16, text: &str) -> Result<(), io::Error> {
        Terminal::move_cursor_to(0, row)?;
        Terminal::clear_line()?;
        Terminal::print(text)?;
        Ok(())
    }

    fn render_welcome_screen(&self) -> Result<(), io::Error> {
        Terminal::move_cursor_to(0, 0)?;
        Terminal::clear_terminal()?;
        let (_col, rows) = Terminal::size();

        for r in 1..rows {
            if r == rows / 3 {
                let message = format!("Welcome to {} - version: {}", NAME, VERSION);

                let terminal_width = Terminal::size().0 as usize;
                let msg_len = message.len();

                let padding = (terminal_width - msg_len) / 2;
                let spaces = " ".repeat(padding - 1);

                let output = format!("~{spaces}{message}");

                self.render_line(r, output.as_str())?;
            } else {
                self.render_line(r, "~")?;
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), io::Error> {
        if !self.need_redraw {
            return Ok(());
        }

        if self.buffer.is_empty() {
            self.render_welcome_screen()?;
        } else {
            self.render_buffer()?;
        }
        self.need_redraw = false;
        Ok(())
    }

    pub fn load(&mut self, file_path: String) -> Result<(), io::Error> {
        self.buffer.load_lines_from_file(file_path)?;
        self.need_redraw = true;
        Ok(())
    }
}
