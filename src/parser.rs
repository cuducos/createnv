use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Stdout, Write};

use anyhow::Result;

use crate::model::Block;

pub struct Parser {
    writer: BufWriter<Stdout>,

    buffer: Option<String>,
    block: Option<Block>,

    written: bool,
}

impl Parser {
    pub fn new(writer: BufWriter<Stdout>) -> Self {
        Self {
            writer,
            buffer: None,
            block: None,
            written: false,
        }
    }

    fn parse_line(&mut self, line: String) -> Result<()> {
        self.buffer = Some(line);
         println!("{}", self.buffer.as_ref().unwrap_or(&"".to_string()));
        Ok(())
    }

    pub fn parse(&mut self, path: String) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            self.parse_line(line?)?;
        }
        self.save_block()?;
        Ok(())
    }

    fn save_block(&mut self) -> Result<()> {
        if let Some(block) = &self.block {
            if self.written {
                write!(self.writer, "\n\n")?;
            }
            write!(self.writer, "{block}")?;
            self.block = None;
        }
        Ok(())
    }
}
