use std::{fs, io};

pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn default() -> Self {
        Buffer { lines: Vec::new() }
    }

    pub fn push(&mut self, val: String) -> Result<(), io::Error> {
        self.lines.push(val);
        Ok(())
    }

    pub fn load_lines_from_file(&mut self, file_name: String) -> Result<(), io::Error> {
        let file = fs::read_to_string(file_name)?;

        for line in file.lines() {
            self.push(line.to_owned())?;
        }
        Ok(())
    }

    pub fn clear_buffer(&mut self) {
        self.lines.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
