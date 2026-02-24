use std::{
    fmt::Display,
    io::{self, Write, stdout},
};

use crossterm::{
    Command, cursor, queue,
    style::Print,
    terminal::{self, Clear, disable_raw_mode, enable_raw_mode},
};

pub struct Terminal {}

impl Terminal {
    pub fn size() -> (u16, u16) {
        terminal::size().unwrap()
    }

    pub fn clear_line() -> Result<(), io::Error> {
        Self::queue_command(Clear(terminal::ClearType::CurrentLine))
    }

    pub fn clear_terminal() -> Result<(), io::Error> {
        Self::queue_command(Clear(terminal::ClearType::All))?;

        Self::queue_command(Clear(terminal::ClearType::Purge))?;
        Self::move_cursor_to(0, 0)?;

        Ok(())
    }

    pub fn print<T: Display>(val: T) -> Result<(), io::Error> {
        Self::queue_command(Print(val))
    }

    pub fn terminate() -> Result<(), io::Error> {
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), io::Error> {
        enable_raw_mode()?;
        Self::clear_terminal()?;
        Self::move_cursor_to(0, 0)?;
        Self::execute()?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), io::Error> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_cursor() -> Result<(), io::Error> {
        Self::queue_command(cursor::Show)
    }

    pub fn move_cursor_to(x: u16, y: u16) -> Result<(), io::Error> {
        Self::queue_command(cursor::MoveTo(x, y))
    }

    pub fn queue_command<T: Command>(command: T) -> Result<(), io::Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn execute() -> Result<(), io::Error> {
        stdout().flush()?;
        Ok(())
    }
}
