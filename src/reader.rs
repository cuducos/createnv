use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{anyhow, Result};

#[derive(PartialEq, Eq)]
pub enum CharType {
    Char(char),
    Eol,
    Eof,
}

pub struct CharReader {
    pub line: usize,
    pub column: usize,
    path: String,
    current_line: Option<String>,
    reader: BufReader<File>,
    done: bool,
}

impl CharReader {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            line: 0,
            column: 0,
            path: path.display().to_string(),
            current_line: None,
            done: false,
            reader: BufReader::new(File::open(path)?),
        })
    }

    pub fn error(&self, character: &CharType, details: Option<String>) -> anyhow::Error {
        let prefix = format!("{}:{}:{}", self.path, self.line, self.column);
        let extra = details.map_or("".to_string(), |msg| format!(": {msg}"));
        let token = match &character {
            CharType::Char(char) => format!("character `{char}`"),
            CharType::Eol => "EOL (end of line)".to_string(),
            CharType::Eof => "EOF (end of file)".to_string(),
        };

        anyhow!(format!("{prefix}: Unexpected {token}{extra}"))
    }

    pub fn next(&mut self) -> Result<CharType> {
        if self.done {
            return Ok(CharType::Eof);
        }
        match &self.current_line {
            None => {
                let mut buffer = "".to_string();
                let size = self.reader.read_line(&mut buffer)?;
                if size == 0 {
                    self.done = true;
                    return Ok(CharType::Eof);
                }
                self.current_line = Some(buffer.clone());
                self.line += 1;
                self.column = 0;
                self.next()
            }
            Some(line) => match line.chars().nth(self.column) {
                Some(char) => match char {
                    '\n' => {
                        self.current_line = None;
                        Ok(CharType::Eol)
                    }
                    _ => {
                        self.column += 1;
                        Ok(CharType::Char(char))
                    }
                },
                None => {
                    self.current_line = None;
                    Ok(CharType::Eol)
                }
            },
        }
    }
}
