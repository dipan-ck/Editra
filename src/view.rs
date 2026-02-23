use std::io;

use crate::{buffer::Buffer, terminal::Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct View {
    buffer: Buffer,
    pub need_redraw: bool,
}

impl View {
    pub fn default() -> Self {
        View {
            buffer: Buffer::default(),
            need_redraw: true,
        }
    }

    fn render_buffer(&self) -> Result<(), io::Error> {
        let height = Terminal::size().1 as usize;
        let width = Terminal::size().0 as usize;

        for curr_row in 0..height {
            if let Some(line) = self.buffer.lines.get(curr_row) {
                self.render_line(curr_row as u16, line.get(0..width).unwrap_or(line))?;
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
