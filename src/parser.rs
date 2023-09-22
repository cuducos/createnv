use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::{
    model::{Block, Comment},
    tokenizer::{Token, Tokenizer},
};

struct Parser {
    tokens: Tokenizer,
    path: String,
    previous_token: Option<Token>,
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(path: &String) -> Result<Self> {
        Ok(Self {
            tokens: Tokenizer::new(PathBuf::from(path))?,
            path: path.clone(),
            current_token: None,
            previous_token: None,
        })
    }

    fn load_next_token(&mut self) -> Result<()> {
        self.previous_token = self.current_token.take();
        match self.tokens.next() {
            Some(token) => self.current_token = Some(token?),
            None => self.current_token = None,
        }

        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Error {
        let prefix = if let Some(curr) = &self.current_token {
            curr.error_prefix(&self.path)
        } else if let Some(prev) = &self.previous_token {
            prev.error_prefix(&self.path)
        } else {
            "EOF".to_string()
        };

        anyhow!("{}: {}", prefix, msg)
    }

    fn parse_title(&mut self) -> Result<String> {
        self.load_next_token()?;
        match self.current_token {
            Some(Token::CommentMark(_, _)) => (),
            Some(_) => return Err(self.error("Expected a title line starting with `#`")),
            None => {
                return Err(
                    self.error("Expected a title line starting with `#`, got the end of the file")
                )
            }
        }

        self.load_next_token()?;
        match &self.current_token {
            Some(Token::Text(_, _, text)) => Ok(text.clone()),
            Some(_) => Err(self.error("Expected the text of the title")),
            None => Err(self.error("Expected the text of the title, got  the end of the file")),
        }
    }

    fn parse_description(&mut self) -> Result<Option<String>> {
        self.load_next_token()?;
        match self.current_token {
            Some(Token::CommentMark(_, _)) => (),
            Some(_) => return Ok(None),
            None => return Err(self.error("Expected a descrition line starting with `#` or a variable definition, got the end of the file")),
        }

        self.load_next_token()?;
        match &self.current_token {
            Some(Token::Text(_, _, text)) => Ok(Some(text.clone())),
            Some(_) => Err(self.error("Expected a descrition text")),
            None => Err(self.error("Expected a descrition text, got the end of the file")),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = vec![];
        let title = Comment::new(self.parse_title()?.as_str());
        let descrition = self
            .parse_description()?
            .map(|desc| Comment::new(desc.as_str()));
        blocks.push(Block::new(title, descrition));

        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".env.sample")
            .into_os_string()
            .into_string();
        let parsed = Parser::new(&sample.unwrap()).unwrap().parse().unwrap();
        let got = parsed
            .iter()
            .map(|block| block.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        let expected = "# Createnv\n# This is a simple example of how Createnv works".to_string();
        assert_eq!(expected, got);
    }
}
//
// TODO: remove (just written for manual tests & debug)
pub fn parser_cli(path: &String) -> Result<()> {
    let mut parser = Parser::new(path)?;
    for block in parser.parse()? {
        println!("{block}");
    }

    Ok(())
}
