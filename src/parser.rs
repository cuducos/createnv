use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{anyhow, Result};

use crate::model::{Block, Comment};

#[derive(PartialEq, Eq)]
pub enum CharType {
    Char(char),
    Eol,
    Eof,
}

const CAPITAL_ASCII_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

struct CharReader {
    line: usize,
    column: usize,
    path: String,
    current_line: Option<String>,
    reader: BufReader<File>,
    done: bool,
}

impl CharReader {
    fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            line: 0,
            column: 0,
            path: path.display().to_string(),
            current_line: None,
            done: false,
            reader: BufReader::new(File::open(path)?),
        })
    }

    fn error(&self, character: &CharType, details: Option<String>) -> anyhow::Error {
        let prefix = format!("{}:{}:{}", self.path, self.line, self.column);
        let extra = details.map_or("".to_string(), |msg| format!(": {msg}"));
        let token = match &character {
            CharType::Char(char) => format!("character `{char}`"),
            CharType::Eol => "EOL (end of line)".to_string(),
            CharType::Eof => "EOF (end of file)".to_string(),
        };

        anyhow!(format!("{prefix}: Unexpected {token}{extra}"))
    }

    fn next(&mut self) -> Result<CharType> {
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

pub struct Parser {
    reader: CharReader,
}

impl Parser {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            reader: CharReader::new(path)?,
        })
    }

    fn check_line_start(&self, char: &CharType) -> Result<()> {
        if self.reader.column != 1 {
            return Ok(());
        }

        match char {
            CharType::Eol => Ok(()),
            CharType::Eof => Ok(()),
            CharType::Char(c) => {
                if *c == '#' || CAPITAL_ASCII_LETTERS.contains(*c) {
                    return Ok(());
                }

                let msg = "A line must start with a capital ASCII letter or `#`".to_string();
                Err(anyhow!(self.reader.error(char, Some(msg))))
            }
        }
    }

    pub fn parse_until(
        &mut self,
        target: Vec<CharType>,
        avoid: Vec<CharType>,
        column: Option<usize>,
    ) -> Result<String> {
        let mut buffer = "".to_string();
        loop {
            let char = self.reader.next()?;
            self.check_line_start(&char)?;
            if avoid.contains(&char) {
                return Err(anyhow!(self.reader.error(&char, None)));
            }
            if target.contains(&char) {
                if let Some(col) = column {
                    if self.reader.column != col {
                        let msg = format!("expected at column {col}");
                        return Err(self.reader.error(&char, Some(msg)));
                    }
                }
                break;
            }
            if let CharType::Char(c) = char {
                buffer.push(c);
            }
        }

        Ok(buffer.to_string())
    }

    fn parse_comment(&mut self) -> Result<Comment> {
        self.parse_until(vec![CharType::Char('#')], vec![CharType::Eol], Some(1))?;
        self.parse_until(vec![CharType::Char(' ')], vec![CharType::Eol], Some(2))?;

        let name = self.parse_until(vec![CharType::Eol], vec![], None)?;

        Ok(Comment::new(name.as_str()))
    }

    fn parse_block(&mut self) -> Result<Block> {
        let title = self
            .parse_comment()
            .map_err(|e| anyhow!("A block must start with `# `: {e}"))?;

        Ok(Block::new(title, None, vec![]))
    }

    pub fn parse(&mut self) -> Result<Vec<Block>> {
        let block = self.parse_block()?;
        let blocks = vec![block];
        Ok(blocks)
    }
}
