use std::path::PathBuf;

use anyhow::Result;

use crate::reader::{CharReader, CharType};

#[derive(Debug, PartialEq)]
enum Token {
    Text(usize, usize, String),
    CommentMark(usize, usize),
    HelpMark(usize, usize),
    EqualSign(usize, usize),
}

impl Token {
    fn error_prefix(&self, path: &String) -> String {
        let (line, column) = match self {
            Token::Text(x, y, _) => (x, y),
            Token::CommentMark(x, y) => (x, y),
            Token::HelpMark(x, y) => (x, y),
            Token::EqualSign(x, y) => (x, y),
        };

        format!("{path}:{line}:{column}")
    }
}

struct Tokenizer {
    reader: CharReader,
    buffer: Option<Token>,
}

impl Tokenizer {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            reader: CharReader::new(path)?,
            buffer: None,
        })
    }

    fn text(&self, buffer: String, eol: bool, prepends_help: bool) -> Token {
        let adjust = match (eol, prepends_help) {
            (true, false) => -1,
            (false, true) => 2,
            _ => 0,
        } + (buffer.len() as i8);

        Token::Text(
            self.reader.line,
            self.reader.column - (adjust as usize),
            buffer.trim().to_string(),
        )
    }

    fn equal_sign(&self) -> Token {
        Token::EqualSign(self.reader.line, self.reader.column)
    }
    fn comment_mark(&self) -> Token {
        Token::CommentMark(self.reader.line, self.reader.column)
    }
    fn help_mark(&self) -> Token {
        Token::HelpMark(self.reader.line, self.reader.column - 2)
    }

    fn next_tokens(&mut self) -> Result<(Option<Token>, Option<Token>)> {
        let mut buffer = "".to_string();
        loop {
            let char = self.reader.next()?;
            match char {
                CharType::Eof => return Ok((None, None)),
                CharType::Eol => {
                    if buffer.is_empty() {
                        continue;
                    }
                    return Ok((Some(self.text(buffer, true, false)), None));
                }
                CharType::Char(c) => {
                    let mut token: Option<Token> = None;
                    let mut prepends_help = false;
                    if c == '=' {
                        token = Some(self.equal_sign());
                    } else if c == '#' {
                        if self.reader.column == 1 {
                            token = Some(self.comment_mark());
                        } else if buffer.ends_with("  ") {
                            buffer = buffer.strip_suffix("  ").unwrap_or("").to_string();
                            prepends_help = true;
                            token = Some(self.help_mark());
                        }
                    }
                    if let Some(t) = token {
                        if buffer.is_empty() {
                            return Ok((Some(t), None));
                        }
                        return Ok((Some(self.text(buffer, false, prepends_help)), Some(t)));
                    }
                    buffer.push(c)
                }
            }
        }
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.buffer.take() {
            return Some(Ok(token));
        }

        match self.next_tokens() {
            Ok((Some(first), Some(second))) => {
                self.buffer = Some(second);
                Some(Ok(first))
            }
            Ok((Some(token), None)) => Some(Ok(token)),
            Ok((None, Some(token))) => Some(Ok(token)),
            Ok((None, None)) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env.sample");
        let tokenizer = Tokenizer::new(sample).unwrap();
        let tokens: Vec<Token> = tokenizer.map(|t| t.unwrap()).collect();
        assert_eq!(tokens.len(), 19);

        // line 1
        assert_eq!(tokens[0], Token::CommentMark(1, 1));
        assert_eq!(tokens[1], Token::Text(1, 2, "Createnv".to_string()));
        assert_eq!(tokens[2], Token::CommentMark(2, 1));

        // line 2
        assert_eq!(
            tokens[3],
            Token::Text(
                2,
                2,
                "This is a simple example of how Createnv works".to_string()
            )
        );

        // line 3
        assert_eq!(tokens[4], Token::Text(3, 1, "NAME".to_string()));
        assert_eq!(tokens[5], Token::EqualSign(3, 5));
        assert_eq!(tokens[6], Token::HelpMark(3, 6));
        assert_eq!(
            tokens[7],
            Token::Text(3, 9, "What's your name?".to_string())
        );

        // line 4
        assert_eq!(tokens[8], Token::Text(4, 1, "GREETING".to_string()));
        assert_eq!(tokens[9], Token::EqualSign(4, 9));
        assert_eq!(tokens[10], Token::Text(4, 10, "Hello, {NAME}!".to_string()));

        // line 5
        assert_eq!(
            tokens[11],
            Token::Text(5, 1, "DO_YOU_LIKE_OPEN_SOURCE".to_string())
        );
        assert_eq!(tokens[12], Token::EqualSign(5, 24));
        assert_eq!(tokens[13], Token::Text(5, 25, "True".to_string()));
        assert_eq!(tokens[14], Token::HelpMark(5, 29));
        assert_eq!(
            tokens[15],
            Token::Text(5, 32, "Do you like open-source?".to_string())
        );

        // line 6
        assert_eq!(tokens[16], Token::Text(6, 1, "PASSWORD".to_string()));
        assert_eq!(tokens[17], Token::EqualSign(6, 9));
        assert_eq!(tokens[18], Token::Text(6, 10, "<random:16>".to_string()));
    }
}

// TODO: remove (just written for manual tests & debug)
pub fn tokenize_cli(path: &String) -> Result<()> {
    for token in Tokenizer::new(PathBuf::from(path))? {
        println!("{:?}", token?);
    }
    if let Some(token) = Tokenizer::new(PathBuf::from(path))?.next() {
        println!(
            "\nThe error prefix looks like:\n{}",
            token?.error_prefix(path)
        );
    }

    Ok(())
}
