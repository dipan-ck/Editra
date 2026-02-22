use std::{
    cmp::min,
    io::{self},
};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::terminal::Terminal;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    quit: bool,
    location: Location,
}

impl Editor {
    pub fn default() -> Self {
        Editor {
            quit: false,
            location: Location { x: 0, y: 0 },
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        Terminal::initialize()?;
        let result = self.render();
        Terminal::terminate()?;
        result
    }

    fn render(&mut self) -> Result<(), io::Error> {
        loop {
            self.refresh_terminal()?;

            if self.quit {
                return Ok(());
            }
            let event = crossterm::event::read()?;
            self.resolve_event(&event)?;
        }
    }
    fn refresh_terminal(&self) -> Result<(), io::Error> {
        Terminal::hide_cursor()?;
        if self.quit {
            Terminal::clear_terminal()?;
            Terminal::print("Goodbye")?;
        } else {
            self.draw_rows()?;
            Terminal::move_cursor_to(self.location.x as u16, self.location.y as u16)?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn calculate_location(&mut self, code: KeyCode) -> Result<(), io::Error> {
        let mut x = self.location.x as u16;
        let mut y = self.location.y as u16;

        let (width, height) = Terminal::size();

        match code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(y.saturating_add(1), height.saturating_sub(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => x = min(x.saturating_add(1), width.saturating_sub(1)),
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            KeyCode::Home => x = 0,
            _ => {}
        }
        self.location = Location {
            x: x as usize,
            y: y as usize,
        };

        Ok(())
    }

    fn resolve_event(&mut self, event: &Event) -> Result<(), io::Error> {
        if let Event::Key(k) = event {
            match k.code {
                KeyCode::Char('q') if k.modifiers == KeyModifiers::CONTROL => {
                    self.quit = true;
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageDown
                | KeyCode::PageUp
                | KeyCode::End
                | KeyCode::Home => self.calculate_location(k.code)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn draw_welcome_message(&self) -> Result<(), io::Error> {
        let message = format!("Welcome to {} - version: {}", NAME, VERSION);

        let terminal_width = Terminal::size().0 as usize;
        let msg_len = message.len();

        let padding = (terminal_width - msg_len) / 2;
        let spaces = " ".repeat(padding - 1);

        let output = format!("~{spaces}{message}");

        Terminal::print(output)
    }

    fn draw_rows(&self) -> Result<(), io::Error> {
        let (_col, rows) = Terminal::size();

        for r in 0..rows {
            if r == rows / 3 {
                Terminal::clear_line()?;
                self.draw_welcome_message()?;
            } else {
                Terminal::move_cursor_to(0, r)?;
                Terminal::clear_line()?;
                Terminal::print("~")?;
            }

            Terminal::print("\r\n")?;
        }

        Ok(())
    }
}
