use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
};

use anyhow::Result;

use crate::model::{Block, Comment};

enum Expecting {
    Title,
    DescriptionOrVariables,
    Variables,
}

struct Parser {
    blocks: Vec<Block>,
    buffer: Option<Block>,
    path: String,
    state: Expecting,
}

impl Parser {
    pub fn new(path: &str) -> Self {
        Self {
            blocks: vec![],
            buffer: None,
            path: path.to_string(),
            state: Expecting::Title,
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        let reader = BufReader::new(File::open(&self.path)?);
        for line in reader.lines() {
            // TODO: empty line => start new block
            match self.state {
                Expecting::Title => {
                    if let Some(txt) = line?.strip_prefix('#') {
                        self.buffer = Some(Block::new(Comment::new(txt.trim()), None));
                    }
                    self.state = Expecting::DescriptionOrVariables;
                }
                Expecting::DescriptionOrVariables => {
                    if let Some(txt) = line?.strip_prefix('#') {
                        if let Some(b) = self.buffer.as_mut() {
                            b.description = Some(Comment::new(txt.trim()));
                        }
                        self.state = Expecting::Variables;
                    } else {
                        // TODO: handle variable line
                    }
                }
                Expecting::Variables => {
                    // TODO: ,
                }
            }
        }
        if let Some(block) = &self.buffer {
            self.blocks.push(block.clone());
            self.buffer = None
        }
        Ok(())
    }
}

// TODO: remove (just written for manual tests & debug)
pub fn parser_cli(path: &str) -> Result<()> {
    let mut parser = Parser::new(path);
    parser.parse()?;
    for block in &mut parser.blocks {
        block.resolve(&mut stdin().lock())?;
        println!("{}", block.as_text()?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, path::PathBuf};

    use super::*;

    #[test]
    fn test_parser() {
        let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".env.sample")
            .into_os_string()
            .into_string();
        let mut parser = Parser::new(&sample.unwrap());
        parser.parse().unwrap();
        let got = parser
            .blocks
            .iter_mut()
            .map(|block| {
                block.resolve(&mut Cursor::new("foobar")).unwrap();
                block.as_text().unwrap()
            })
            .collect::<Vec<String>>()
            .join("\n");

        // TODO: include variables (currently it is just title and description)
        let expected = "# Createnv\n# This is a simple example of how Createnv works".to_string();

        assert_eq!(expected, got);
    }
}
