use std::{
    cmp::min,
    env,
    io::{self},
};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::{terminal::Terminal, view::View};

struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    quit: bool,
    location: Location,
    view: View,
}

impl Editor {
    pub fn default() -> Self {
        Editor {
            quit: false,
            view: View::default(),
            location: Location { x: 0, y: 0 },
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
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.quit = true;
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageDown
                | KeyCode::PageUp
                | KeyCode::End
                | KeyCode::Home => self.calculate_location(key_event.code)?,

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
