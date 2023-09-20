use std::path::PathBuf;

use anyhow::Result;

use crate::reader::{CharReader, CharType};

#[derive(Debug)]
pub enum Token {
    Text(String),
    CommentMark,
    HelpMark,
    EqualSign,
}

pub struct Tokenizer {
    reader: CharReader,
}

impl Tokenizer {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            reader: CharReader::new(path)?,
        })
    }

    fn next_tokens(&mut self) -> Result<Vec<Token>> {
        let mut buffer = "".to_string();
        loop {
            let char = self.reader.next()?;
            match char {
                CharType::Eof => return Ok(vec![]),
                CharType::Eol => {
                    if buffer.is_empty() {
                        continue;
                    }
                    return Ok(vec![Token::Text(buffer.trim().to_string())]);
                }
                CharType::Char(c) => {
                    let mut token: Option<Token> = None;
                    if c == '=' {
                        token = Some(Token::EqualSign);
                    } else if c == '#' && self.reader.column == 1 {
                        token = Some(Token::CommentMark);
                    } else if c == ' ' && buffer.ends_with("  #") {
                        buffer = buffer.strip_suffix("  #").unwrap_or("").to_string();
                        token = Some(Token::HelpMark);
                    }
                    if let Some(t) = token {
                        if buffer.is_empty() {
                            return Ok(vec![t]);
                        }
                        return Ok(vec![Token::Text(buffer.trim().to_string()), t]);
                    }
                    buffer.push(c)
                }
            }
        }
    }

    // TODO: make iterator?
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens: Vec<Token> = vec![];
        loop {
            let new_tokens = self.next_tokens()?;
            if new_tokens.is_empty() {
                break;
            }
            tokens.extend(new_tokens);
        }
        Ok(tokens)
    }
}
