use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::Result;

enum CharType {
    Char(char),
    Eol,
    Eof,
}

struct CharReader {
    line: usize,
    column: usize,
    current_line: Option<String>,
    reader: BufReader<File>,
    done: bool,
}

impl CharReader {
    fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            line: 0,
            column: 0,
            current_line: None,
            done: false,
            reader: BufReader::new(File::open(path)?),
        })
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

    pub fn parse(&mut self) -> Result<()> {
        let mut copy = "".to_string();
        loop {
            match self.reader.next()? {
                CharType::Char(char) => copy.push(char),
                CharType::Eol => copy += "\n",
                CharType::Eof => break,
            }
        }
        println!("{}", copy);
        Ok(())
    }
}
