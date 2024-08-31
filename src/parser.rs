use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use rand::{thread_rng, Rng};
use regex::Regex;

const RANDOM_VARIABLE_PATTERN: &str = r"\<random(:(?P<size>\d*))?\>";
const AUTO_GENERATED_PATTERN: &str = r"\{[A-Z0-9_]+\}";

use crate::model::{AutoGeneratedVariable, Block, Comment, SimpleVariable, VariableType};

enum Expecting {
    Title,
    DescriptionOrVariables,
    Variables,
}

pub struct Parser {
    path: String,
    random_chars: String,
    use_default: bool,
    random_pattern: Regex,
    auto_generated_pattern: Regex,
    state: Expecting,
    buffer: Option<Block>,
    pub blocks: Vec<Block>,
}

impl Parser {
    pub fn new(path: &str, random_chars: &str, use_default: &bool) -> Result<Self> {
        Ok(Self {
            path: path.to_string(),
            random_chars: random_chars.to_string(),
            use_default: *use_default,
            random_pattern: Regex::new(RANDOM_VARIABLE_PATTERN)?,
            auto_generated_pattern: Regex::new(AUTO_GENERATED_PATTERN)?,
            state: Expecting::Title,
            buffer: None,
            blocks: vec![],
        })
    }

    fn flush<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        if let Some(block) = self.buffer.as_mut() {
            block.resolve(terminal, self.use_default)?;
            self.blocks.push(block.clone());
            self.buffer = None
        }
        Ok(())
    }

    fn parse_random_variable(
        &self,
        name: &str,
        description: Option<&str>,
        value: &str,
    ) -> Result<SimpleVariable> {
        if let Some(matches) = self.random_pattern.captures(value) {
            let mut rng = thread_rng();
            let length = matches
                .name("size")
                .map(|m| m.as_str().parse::<usize>())
                .transpose()?
                .unwrap_or(rng.gen_range(64..=128));
            let max_chars_idx = self.random_chars.chars().count();
            let mut value: String = String::from("");
            for _ in 0..length {
                let pos = rng.gen_range(0..max_chars_idx);
                value.push(self.random_chars.chars().nth(pos).unwrap())
            }
            Ok(SimpleVariable::new(name, Some(value.as_str()), description))
        } else {
            Err(anyhow::anyhow!("Invalid random variable: {}", value))
        }
    }

    fn parse_auto_generated_variable(
        &self,
        name: &str,
        value: &str,
    ) -> Result<AutoGeneratedVariable> {
        if self.auto_generated_pattern.find(value).is_some() {
            return Ok(AutoGeneratedVariable::new(name, value));
        }
        Err(anyhow::anyhow!(
            "Invalid auto-generated variable: {}",
            value
        ))
    }

    fn parse_variable(&self, line: &str) -> Result<VariableType> {
        let (name, rest) = line
            .split_once('=')
            .ok_or(anyhow::anyhow!("Invalid variable line: {}", line))?;
        let (mut default, description) = match rest.split_once("  # ") {
            Some((default, help)) => (Some(default), Some(help)),
            None => (Some(rest), None),
        };
        if let Some(val) = default {
            if val.is_empty() {
                default = None;
            } else {
                if let Ok(v) = self.parse_random_variable(name, description, val) {
                    return Ok(VariableType::Input(v));
                }
                if let Ok(v) = self.parse_auto_generated_variable(name, val) {
                    return Ok(VariableType::AutoGenerated(v));
                }
            }
        }
        let variable = SimpleVariable::new(name, default, description);
        Ok(VariableType::Input(variable))
    }

    pub fn parse<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        let reader = BufReader::new(File::open(&self.path)?);
        for line in reader.lines() {
            let cleaned = line?.trim().to_string();
            if cleaned.is_empty() {
                self.flush(terminal)?;
                self.state = Expecting::Title;
                continue;
            }
            match self.state {
                Expecting::Title => {
                    if let Some(txt) = cleaned.strip_prefix('#') {
                        self.buffer = Some(Block::new(Comment::new(txt.trim()), None));
                    }
                    self.state = Expecting::DescriptionOrVariables;
                }
                Expecting::DescriptionOrVariables => {
                    if let Some(txt) = cleaned.strip_prefix('#') {
                        if let Some(b) = self.buffer.as_mut() {
                            b.description = Some(Comment::new(txt.trim()));
                        }
                        self.state = Expecting::Variables;
                    } else {
                        let variable = self.parse_variable(&cleaned)?;
                        if let Some(b) = self.buffer.as_mut() {
                            b.variables.push(variable);
                        }
                    }
                }
                Expecting::Variables => {
                    let variable = self.parse_variable(&cleaned)?;
                    if let Some(b) = self.buffer.as_mut() {
                        b.variables.push(variable);
                    }
                }
            }
        }
        self.flush(terminal)?;
        Ok(())
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for block in &self.blocks {
            if !first {
                writeln!(f)?;
            }
            write!(f, "{}", block)?;
            first = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, path::PathBuf};

    use super::*;
    use crate::DEFAULT_RANDOM_CHARS;

    #[test]
    fn test_parser() {
        let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".env.sample")
            .into_os_string()
            .into_string();
        let mut parser = Parser::new(&sample.unwrap(), DEFAULT_RANDOM_CHARS, &false).unwrap();
        parser.parse(&mut Cursor::new("World")).unwrap();
        assert_eq!(parser.blocks.len(), 1);
        assert_eq!(parser.blocks[0].variables.len(), 4);
        let names: [&str; 4] = ["NAME", "GREETING", "DO_YOU_LIKE_OPEN_SOURCE", "PASSWORD"];
        for (variable, expected) in parser.blocks[0].variables.iter().zip(names) {
            let got = match variable {
                VariableType::Input(v) => &v.name,
                VariableType::AutoGenerated(v) => &v.name,
            };
            assert_eq!(got, expected);
        }
        for (idx, variable) in parser.blocks[0].variables.iter().enumerate() {
            if idx != 1 {
                assert!(
                    matches!(variable, VariableType::Input(_)),
                    "Expected variable number {} to be Input, got {:?}",
                    idx + 1,
                    variable
                );
            }
            if idx == 1 {
                assert!(
                    matches!(variable, VariableType::AutoGenerated(_)),
                    "Expected variable 2 to be AutoGenerated, got {:?}",
                    variable
                );
            }
        }
    }
}
