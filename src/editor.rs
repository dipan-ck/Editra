use std::io;

use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

pub struct Editor {
    quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { quit: false }
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        self.prepare_terminal()?;
        let result = self.render();
        self.terminate()?;
        result
    }

    fn prepare_terminal(&self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        self.clear_terminal()
    }

    fn clear_terminal(&self) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All))
    }

    fn terminate(&self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        Ok(())
    }

    fn render(&mut self) -> Result<(), io::Error> {
        loop {
            let event = crossterm::event::read()?;
            self.resolve_event(&event)?;

            self.refresh_terminal()?;

            if self.quit {
                return Ok(());
            }
        }
    }

    fn refresh_terminal(&self) -> Result<(), io::Error> {
        if self.quit {
            self.clear_terminal()?;
            print!("Goodbye..\r\n");
        }

        Ok(())
    }

    fn resolve_event(&mut self, event: &Event) -> Result<(), io::Error> {
        if let Event::Key(k) = event {
            match k.code {
                KeyCode::Char('q') if k.modifiers == KeyModifiers::CONTROL => {
                    self.quit = true;
                }
                _ => {
                    println!("{}", k.code);
                }
            }
        }

        Ok(())
    }
}
