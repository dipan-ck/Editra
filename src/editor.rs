use std::{
    env,
    io::{self},
};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::{terminal::Terminal, view::View};

pub struct Editor {
    quit: bool,
    pub view: View,
}

impl Editor {
    pub fn default() -> Self {
        Editor {
            quit: false,
            view: View::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        Terminal::initialize()?;
        self.handle_args()?;
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
    fn refresh_terminal(&mut self) -> Result<(), io::Error> {
        Terminal::hide_cursor()?;
        if self.quit {
            Terminal::clear_terminal()?;
            Terminal::print("Goodbye")?;
        } else {
            self.view.render()?;
            let (x, y) = self.view.get_cursor_location();
            Terminal::move_cursor_to(x as u16, y as u16)?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn resolve_event(&mut self, event: &Event) -> Result<(), io::Error> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.quit = true;
                }

                KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => {
                    let old_file_type = self.view.buffer.file_type;
                    self.view.buffer.save_buffer_as_file()?;
                    if old_file_type != self.view.buffer.file_type {
                        self.view
                            .highlighter
                            .update_file_type(self.view.buffer.file_type);
                        self.view.highlighter.highlight_all(&self.view.buffer.lines);
                        self.view.need_redraw = true;
                    }
                }

                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageDown
                | KeyCode::PageUp
                | KeyCode::End
                | KeyCode::Home => self.view.update_cursor_location(key_event.code)?,
                KeyCode::Char(c) => self.view.insert_char_to_line(c)?,
                KeyCode::Delete => self.view.delete_char()?,
                KeyCode::Backspace => self.view.backspace_char()?,
                KeyCode::Tab => self.view.insert_tab()?,
                KeyCode::Enter => self.view.insert_newline()?,

                _ => {}
            },

            Event::Resize(_w, _h) => {
                self.view.need_redraw = true;
            }

            _ => {}
        }

        Ok(())
    }

    fn handle_args(&mut self) -> Result<(), io::Error> {
        let args: Vec<String> = env::args().collect();

        if let Some(path) = args.get(1) {
            self.view.load(path.to_owned())?;
        }

        Ok(())
    }
}
