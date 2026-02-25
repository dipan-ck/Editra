use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightType {
    None,
    Number,
    Keyword,
    Type,
    Literal,
    Character,
    Lifetime,
    Comment,
    String,
}

impl HighlightType {
    pub fn to_color(&self) -> crossterm::style::Color {
        use crossterm::style::Color;
        match self {
            HighlightType::Number => Color::Red,
            HighlightType::Keyword => Color::Blue,
            HighlightType::Type => Color::Green,
            HighlightType::Literal => Color::Magenta,
            HighlightType::Character => Color::Rgb {
                r: 255,
                g: 191,
                b: 0,
            },
            HighlightType::Lifetime => Color::Cyan,
            HighlightType::Comment => Color::DarkGreen,
            HighlightType::String => Color::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
            HighlightType::None => Color::Reset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub start: usize,
    pub end: usize,
    pub highlight_type: HighlightType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    PlainText,
    Rust,
}

impl FileType {
    pub fn from_filename(filename: &str) -> Self {
        if filename.ends_with(".rs") {
            FileType::Rust
        } else {
            FileType::PlainText
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FileType::PlainText => "Text",
            FileType::Rust => "Rust",
        }
    }
}

trait SyntaxHighlighter {
    fn highlight(&self, line: &str) -> Vec<Annotation>;
}

struct PlainTextHighlighter;

impl SyntaxHighlighter for PlainTextHighlighter {
    fn highlight(&self, _line: &str) -> Vec<Annotation> {
        Vec::new()
    }
}

impl Default for PlainTextHighlighter {
    fn default() -> Self {
        Self
    }
}

struct RustSyntaxHighlighter {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    literals: Vec<&'static str>,
}

impl RustSyntaxHighlighter {
    fn new() -> Self {
        Self {
            keywords: vec![
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "self", "Self", "struct", "super", "trait",
                "type", "unsafe", "use", "where", "while",
            ],
            types: vec![
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char", "String", "str", "Vec", "HashMap", "Option",
                "Result",
            ],
            literals: vec!["Some", "None", "Ok", "Err", "true", "false"],
        }
    }

    fn is_number(word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        // Hex, binary, octal
        if word.len() >= 3 {
            let prefix = &word[..2];
            if (prefix == "0x" || prefix == "0X")
                && word[2..].chars().all(|c| c.is_ascii_hexdigit() || c == '_')
                && !word.starts_with("0x_")
                && !word.ends_with('_')
            {
                return true;
            }
            if (prefix == "0b" || prefix == "0B")
                && word[2..].chars().all(|c| c == '0' || c == '1' || c == '_')
                && !word.starts_with("0b_")
                && !word.ends_with('_')
            {
                return true;
            }
            if (prefix == "0o" || prefix == "0O")
                && word[2..]
                    .chars()
                    .all(|c| ('0'..='7').contains(&c) || c == '_')
                && !word.starts_with("0o_")
                && !word.ends_with('_')
            {
                return true;
            }
        }

        // Regular numbers (int, float, scientific)
        let mut has_dot = false;
        let mut has_e = false;
        let mut after_e = false;

        for (i, c) in word.chars().enumerate() {
            if c == '.' {
                if has_dot || after_e || i == 0 || i == word.len() - 1 {
                    return false;
                }
                has_dot = true;
            } else if c == 'e' || c == 'E' {
                if has_e || i == 0 || i == word.len() - 1 {
                    return false;
                }
                has_e = true;
                after_e = true;
            } else if c == '_' {
                if i == 0 || i == word.len() - 1 {
                    return false;
                }
            } else if !c.is_ascii_digit() {
                return false;
            }

            if after_e && c != 'e' && c != 'E' && c == '.' {
                return false;
            }
        }

        true
    }
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&self, line: &str) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        let _byte_index = 0;

        // Check for comments first
        if let Some(pos) = line.find("//") {
            annotations.push(Annotation {
                start: pos,
                end: line.len(),
                highlight_type: HighlightType::Comment,
            });
            return annotations;
        }

        // Simple word-based highlighting
        for (start, word) in line.split_word_bound_indices() {
            if word.chars().all(|c| c.is_whitespace()) {
                continue;
            }

            if self.keywords.contains(&word) {
                annotations.push(Annotation {
                    start,
                    end: start + word.len(),
                    highlight_type: HighlightType::Keyword,
                });
            } else if self.types.contains(&word) {
                annotations.push(Annotation {
                    start,
                    end: start + word.len(),
                    highlight_type: HighlightType::Type,
                });
            } else if self.literals.contains(&word) {
                annotations.push(Annotation {
                    start,
                    end: start + word.len(),
                    highlight_type: HighlightType::Literal,
                });
            } else if Self::is_number(word) {
                annotations.push(Annotation {
                    start,
                    end: start + word.len(),
                    highlight_type: HighlightType::Number,
                });
            } else if word.starts_with('"') {
                // Simple string highlighting
                if let Some(end_pos) = line[start + 1..].find('"') {
                    annotations.push(Annotation {
                        start,
                        end: start + end_pos + 2,
                        highlight_type: HighlightType::String,
                    });
                }
            } else if word.starts_with('\'') && word.len() >= 2 {
                // Simple char highlighting
                annotations.push(Annotation {
                    start,
                    end: start + word.len().min(3),
                    highlight_type: HighlightType::Character,
                });
            }
        }

        annotations
    }
}

impl Default for RustSyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Highlighter {
    syntax_highlighter: Box<dyn SyntaxHighlighter>,
    annotations: HashMap<usize, Vec<Annotation>>,
}

impl Highlighter {
    pub fn new(file_type: FileType) -> Self {
        Self {
            syntax_highlighter: Self::create_syntax_highlighter(file_type),
            annotations: HashMap::new(),
        }
    }

    fn create_syntax_highlighter(file_type: FileType) -> Box<dyn SyntaxHighlighter> {
        match file_type {
            FileType::Rust => Box::<RustSyntaxHighlighter>::default(),
            FileType::PlainText => Box::<PlainTextHighlighter>::default(),
        }
    }

    pub fn update_file_type(&mut self, file_type: FileType) {
        self.syntax_highlighter = Self::create_syntax_highlighter(file_type);
        self.annotations.clear();
    }

    pub fn highlight_all(&mut self, lines: &[String]) {
        self.annotations.clear();
        for (line_idx, line) in lines.iter().enumerate() {
            let annotations = self.syntax_highlighter.highlight(line);
            if !annotations.is_empty() {
                self.annotations.insert(line_idx, annotations);
            }
        }
    }

    pub fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.annotations.get(&line_idx)
    }

    pub fn invalidate_from(&mut self, start_line: usize, lines: &[String]) {
        for line_idx in start_line..lines.len() {
            let annotations = self.syntax_highlighter.highlight(&lines[line_idx]);
            if !annotations.is_empty() {
                self.annotations.insert(line_idx, annotations);
            } else {
                self.annotations.remove(&line_idx);
            }
        }
    }
}
