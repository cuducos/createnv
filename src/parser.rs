use anyhow::{anyhow, Result};

use crate::{
    model::{Block, Comment},
    reader::{CharReader, CharType},
};

const CAPITAL_ASCII_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct Parser {
    reader: CharReader,
}

impl Parser {
    pub fn new(path: std::path::PathBuf) -> Result<Self> {
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
