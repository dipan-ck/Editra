use std::io::{self};

use crate::highlighter::{Annotation, Highlighter};
use crossterm::event::KeyCode;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
    pub buffer: Buffer,
    pub need_redraw: bool,
    pub cursor_location: CursorLocation,
    pub offset: Offset,
    pub highlighter: Highlighter,
}

impl View {
    pub fn default() -> Self {
        View {
            buffer: Buffer::default(),
            need_redraw: true,
            cursor_location: CursorLocation { x: 0, y: 0 },
            offset: Offset { x: 0, y: 0 },
            highlighter: Highlighter::new(crate::highlighter::FileType::PlainText),
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
                    if line.graphemes(true).count() < x {
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
                    if line.graphemes(true).count() < x {
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
                    if x < line.graphemes(true).count() {
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
        let height = Terminal::size().1 as usize - 1;
        let width = Terminal::size().0 as usize;

        for curr_row in 0..height {
            if let Some(line) = self.buffer.lines.get(curr_row + self.offset.y) {
                let mut current_width = 0;
                let mut skipped_width = 0;

                let mut visible = String::new();

                for g in line.graphemes(true) {
                    let w = g.width();

                    // skip until horizontal offset reached
                    if skipped_width < self.offset.x {
                        skipped_width += w;
                        continue;
                    }

                    // stop if screen full
                    if current_width + w > width {
                        break;
                    }

                    visible.push_str(g);
                    current_width += w;
                }
                self.render_line(
                    curr_row as u16,
                    visible.as_str(),
                    Some(curr_row + self.offset.y),
                )?;
            } else {
                self.render_line(curr_row as u16, "~", None)?;
            }
        }

        Ok(())
    }

    fn render_line(&self, row: u16, text: &str, line_idx: Option<usize>) -> Result<(), io::Error> {
        Terminal::move_cursor_to(0, row)?;
        Terminal::clear_line()?;

        if let Some(idx) = line_idx {
            if let Some(annotations) = self.highlighter.get_annotations(idx) {
                return self.render_line_with_highlighting(text, annotations);
            }
        }

        Terminal::print(text)?;
        Ok(())
    }

    fn render_line_with_highlighting(
        &self,
        text: &str,
        annotations: &[Annotation],
    ) -> Result<(), io::Error> {
        let mut last_end = 0;
        let mut sorted = annotations.to_vec();
        sorted.sort_by_key(|a| a.start);

        for annotation in sorted {
            if annotation.start > last_end {
                Terminal::print(&text[last_end..annotation.start])?;
            }

            let color = annotation.highlight_type.to_color();
            Terminal::queue_command(SetForegroundColor(color))?;
            Terminal::print(&text[annotation.start..annotation.end])?;
            Terminal::queue_command(ResetColor)?;

            last_end = annotation.end;
        }

        if last_end < text.len() {
            Terminal::print(&text[last_end..])?;
        }

        Ok(())
    }

    pub fn insert_char_to_line(&mut self, c: char) -> Result<(), io::Error> {
        if self.cursor_location.y >= self.buffer.lines.len() {
            self.buffer.lines.push(String::new());
        }

        let line = self.buffer.lines.get_mut(self.cursor_location.y).unwrap();

        let byte_index = line
            .grapheme_indices(true)
            .nth(self.cursor_location.x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        line.insert(byte_index, c);

        self.cursor_location.x += 1;
        self.need_redraw = true;

        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<(), io::Error> {
        let y = self.cursor_location.y;
        let x = self.cursor_location.x;

        if y >= self.buffer.lines.len() {
            return Ok(());
        }

        let grapheme_count = self.buffer.lines[y].graphemes(true).count();

        // CASE: merge next line
        if x >= grapheme_count {
            if y + 1 >= self.buffer.lines.len() {
                return Ok(()); // end of document
            }

            // remove next line FIRST (no borrow active)
            let next_line = self.buffer.lines.remove(y + 1);

            // now borrow current line
            self.buffer.lines[y].push_str(&next_line);

            self.need_redraw = true;
            return Ok(());
        }

        // normal delete
        let line = &mut self.buffer.lines[y];

        let start = line.grapheme_indices(true).nth(x).map(|(i, _)| i);

        let end = line
            .grapheme_indices(true)
            .nth(x + 1)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        if let Some(start_index) = start {
            line.replace_range(start_index..end, "");
        }

        self.need_redraw = true;

        Ok(())
    }

    pub fn backspace_char(&mut self) -> Result<(), io::Error> {
        let y = self.cursor_location.y;
        let x = self.cursor_location.x;

        if y == 0 && x == 0 {
            return Ok(());
        }

        if x == 0 {
            let current_line = self.buffer.lines.remove(y);

            let prev_line = self.buffer.lines.get_mut(y - 1).unwrap();

            let prev_len = prev_line.graphemes(true).count();

            prev_line.push_str(&current_line);

            self.cursor_location.y -= 1;
            self.cursor_location.x = prev_len;

            self.need_redraw = true;
            self.buffer.modified = true;

            self.highlighter.invalidate_from(y - 1, &self.buffer.lines);

            return Ok(());
        }

        if y >= self.buffer.lines.len() {
            return Ok(());
        }

        let line_len = self.buffer.lines[y].graphemes(true).count();
        if x > line_len {
            return Ok(());
        }

        self.cursor_location.x -= 1;
        self.delete_char()?;

        Ok(())
    }

    pub fn insert_tab(&mut self) -> Result<(), io::Error> {
        self.insert_char_to_line('\t')
    }

    pub fn insert_newline(&mut self) -> Result<(), io::Error> {
        let y = self.cursor_location.y;
        let x = self.cursor_location.x;

        // Case: cursor below document
        if y >= self.buffer.lines.len() {
            self.buffer.lines.push(String::new());
            self.cursor_location.y += 1;
            self.cursor_location.x = 0;
            return Ok(());
        }

        let line = self.buffer.lines.get_mut(y).unwrap();

        let byte_index = line
            .grapheme_indices(true)
            .nth(x)
            .map(|(i, _)| i)
            .unwrap_or(line.len());

        // split line
        let new_line = line.split_off(byte_index);

        // insert new line below
        self.buffer.lines.insert(y + 1, new_line);

        // move cursor
        self.cursor_location.y += 1;
        self.cursor_location.x = 0;

        self.need_redraw = true;

        Ok(())
    }

    fn render_welcome_screen(&self) -> Result<(), io::Error> {
        Terminal::move_cursor_to(0, 0)?;
        Terminal::clear_terminal()?;
        let (_col, rows) = Terminal::size();
        let rows = rows - 1;

        for r in 1..rows {
            if r == rows / 3 {
                let message = format!("Welcome to {} - version: {}", NAME, VERSION);

                let terminal_width = Terminal::size().0 as usize;
                let msg_len = message.len();

                let padding = (terminal_width - msg_len) / 2;
                let spaces = " ".repeat(padding - 1);

                let output = format!("~{spaces}{message}");
                self.render_line(r, output.as_str(), None)?;
            } else {
                self.render_line(r, "~", None)?;
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

        self.render_status_bar()?;
        self.need_redraw = false;
        Ok(())
    }

    pub fn load(&mut self, file_path: String) -> Result<(), io::Error> {
        self.buffer.load_lines_from_file(file_path)?;
        self.highlighter.update_file_type(self.buffer.file_type);
        self.highlighter.highlight_all(&self.buffer.lines);
        self.need_redraw = true;
        Ok(())
    }

    fn render_status_bar(&self) -> Result<(), io::Error> {
        let (width, height) = Terminal::size();
        let height = height as usize;

        // Move to status bar position (second to last row)
        Terminal::move_cursor_to(0, (height - 1) as u16)?;
        Terminal::clear_line()?;

        // Set inverted colors for status bar
        Terminal::queue_command(SetAttribute(Attribute::Reverse))?;

        // Left side: filename or [No Name]
        let filename = self
            .buffer
            .file_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("[No Name]");

        // Middle: file type
        let file_type = self.buffer.file_type.name();

        // Right side: cursor position and line count
        let line_count = self.buffer.lines.len();
        let current_line = self.cursor_location.y + 1; // 1-indexed for display
        let current_col = self.cursor_location.x + 1;

        let right_status = format!(
            "{} | {}/{} | Ln {}, Col {}",
            file_type,
            current_line,
            line_count.max(1),
            current_line,
            current_col
        );

        // Calculate spacing
        let left_len = filename.len();
        let right_len = right_status.len();
        let padding = width as usize - left_len - right_len;

        // Print status bar
        Terminal::print(filename)?;
        Terminal::print(" ".repeat(padding))?;
        Terminal::print(&right_status)?;

        // Reset attributes
        Terminal::queue_command(SetAttribute(Attribute::Reset))?;

        Ok(())
    }
}
