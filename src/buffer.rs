use std::{
    fs::{self, File},
    io::{self, Write},
};

use crate::highlighter::FileType;

pub struct Buffer {
    pub lines: Vec<String>,
    pub file_name: Option<String>,
    pub file_type: FileType,
    pub modified: bool,
}

impl Buffer {
    pub fn default() -> Self {
        Buffer {
            lines: Vec::new(),
            file_name: None,
            file_type: FileType::PlainText,
            modified: false,
        }
    }

    pub fn push(&mut self, val: String) -> Result<(), io::Error> {
        self.lines.push(val);
        Ok(())
    }

    pub fn load_lines_from_file(&mut self, file_name: String) -> Result<(), io::Error> {
        self.file_type = FileType::from_filename(&file_name);
        self.file_name = Some(file_name.to_owned());
        let file = fs::read_to_string(file_name)?;

        for line in file.lines() {
            self.push(line.to_owned())?;
        }
        Ok(())
    }

    pub fn clear_buffer(&mut self) {
        self.lines.clear();
    }

    pub fn save_buffer_as_file(&mut self) -> Result<(), io::Error> {
        if let Some(path) = &self.file_name {
            let mut file = File::create(path)?;
            self.file_type = FileType::from_filename(path);
            for line in &self.lines {
                writeln!(file, "{}", line)?;
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
