use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use rand::{thread_rng, Rng};

use crate::model::{AutoGeneratedVariable, Block, Comment, SimpleVariable, VariableType};

const FIRST_CHAR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NAME_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
const RANDOM_VARIABLE_PREFIX: &str = "<random";
const HELP_TITLE: &str = "This is the first line of a block. A block is a \
    group of lines separated from others by one (or more) empty line(s). \
    The first line of a block is expected to be a title, that is to say, to \
    start with `# `, the remaining text is considered the title of this block. \
    This line does not match  this pattern.";
const HELP_DESCRIPTION: &str = "This is the second line of a block. A block is \
    a group of lines separated from others by one (or more) empty line(s). The \
    second line of a block is expected to be a description of that block or a \
    variable line. The description line should start `# `, and the remaining  \
    text is considered the description of this block. A config variable line \
    should start with a name in uppercase, no spaces, followed by an equal \
    sign. No spaces before the equal sign. This lines does not match this \
    expected patterns.";
const HELP_VARIABLE: &str = "This line was expected to be a variable line. The \
    format should be a name using capital ASCII letters, digits or underscore, \
    followed by an equal sign. No spaces before the equal sign. This line does \
    not match this expected pattern.";

fn is_valid_name(name: &str) -> bool {
    match name.chars().next() {
        Some(c) => {
            if !FIRST_CHAR.contains(c) {
                return false;
            }
        }
        None => return false,
    }
    for c in name.chars() {
        if !NAME_CHARS.contains(c) {
            return false;
        }
    }
    true
}

fn is_auto_generated_variable(value: &str) -> bool {
    if let Some(first) = value.find('{') {
        if let Some(second) = value[first + 1..].find('}') {
            let name = &value[first + 1..first + second];
            return is_valid_name(name);
        }
    }
    false
}

fn is_random_variable(value: &str) -> (bool, Option<usize>) {
    if let Some(rest) = value.strip_prefix(RANDOM_VARIABLE_PREFIX) {
        if let Some(number) = rest.strip_suffix('>') {
            if number.is_empty() {
                return (true, None);
            } else if !number.starts_with(':') {
                return (false, None);
            } else if let Ok(n) = number[1..].parse() {
                return (true, Some(n));
            }
        }
    }
    (false, None)
}

enum Expecting {
    Title,
    DescriptionOrVariables,
    Variables,
}

impl Display for Expecting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expecting::Title => write!(f, "expecting a block title"),
            Expecting::DescriptionOrVariables => {
                write!(f, "expecting a block description or a variable line")
            }
            Expecting::Variables => write!(f, "expecting a variable line"),
        }
    }
}

pub struct Parser {
    path: String,
    random_chars: String,
    use_default: bool,
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
            state: Expecting::Title,
            buffer: None,
            blocks: vec![],
        })
    }

    fn parse_random_variable(
        &self,
        name: &str,
        description: Option<&str>,
        value: &str,
    ) -> Option<SimpleVariable> {
        let (is_random, size) = is_random_variable(value);
        if is_random {
            let mut rng = thread_rng();
            let length = size.unwrap_or(rng.gen_range(64..=128));
            let max_chars_idx = self.random_chars.chars().count();
            let mut value: String = String::from("");
            for _ in 0..length {
                let pos = rng.gen_range(0..max_chars_idx);
                value.push(self.random_chars.chars().nth(pos).unwrap())
            }
            return Some(SimpleVariable::new(name, Some(value.as_str()), description));
        }
        None
    }

    fn parse_auto_generated_variable(
        &self,
        name: &str,
        value: &str,
    ) -> Option<AutoGeneratedVariable> {
        if is_auto_generated_variable(value) {
            return Some(AutoGeneratedVariable::new(name, value));
        }
        None
    }

    fn parse_variable(&self, pos: usize, line: &str) -> Result<VariableType> {
        let (name, rest) = line.split_once('=').ok_or(anyhow::anyhow!(
            "Invalid variable line on line {}: {}\nHint: {}",
            pos,
            line,
            HELP_VARIABLE
        ))?;
        if !is_valid_name(name) {
            return Err(anyhow::anyhow!(
                "Invalid variable name on line {}: {}\nHint :{}",
                pos,
                name,
                HELP_VARIABLE
            ));
        }
        let (mut default, description) = match rest.split_once("  # ") {
            Some((default, help)) => (Some(default), Some(help)),
            None => (Some(rest), None),
        };
        if let Some(val) = default {
            if val.is_empty() {
                default = None;
            } else {
                if let Some(v) = self.parse_random_variable(name, description, val) {
                    return Ok(VariableType::Input(v));
                }
                if let Some(v) = self.parse_auto_generated_variable(name, val) {
                    return Ok(VariableType::AutoGenerated(v));
                }
            }
        }
        let variable = SimpleVariable::new(name, default, description);
        Ok(VariableType::Input(variable))
    }

    fn flush<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        if let Some(block) = self.buffer.as_mut() {
            block.resolve(terminal, self.use_default)?;
            self.blocks.push(block.clone());
            self.buffer = None
        }
        Ok(())
    }

    pub fn parse<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        let reader = BufReader::new(File::open(&self.path)?);
        let mut cursor: usize = 0;
        for (idx, line) in reader.lines().enumerate() {
            cursor = idx + 1;
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
                        self.state = Expecting::DescriptionOrVariables;
                    } else {
                        return Err(anyhow::anyhow!(
                            "Unexpected title on line {}: {}\nHint: {}",
                            cursor,
                            cleaned,
                            HELP_TITLE
                        ));
                    }
                }
                Expecting::DescriptionOrVariables => {
                    if let Some(txt) = cleaned.strip_prefix('#') {
                        if let Some(b) = self.buffer.as_mut() {
                            b.description = Some(Comment::new(txt.trim()));
                        }
                        self.state = Expecting::Variables;
                    } else {
                        let variable = self.parse_variable(cursor, &cleaned)?;
                        if let Some(b) = self.buffer.as_mut() {
                            b.variables.push(variable);
                        }
                    }
                }
                Expecting::Variables => {
                    let variable = self.parse_variable(cursor, &cleaned)?;
                    if let Some(b) = self.buffer.as_mut() {
                        b.variables.push(variable);
                    }
                }
            }
        }
        let last_block_has_variables = self
            .buffer
            .as_ref()
            .map(|block| !block.variables.is_empty())
            .unwrap_or(false);
        if !last_block_has_variables {
            let help = match self.state {
                Expecting::Title => HELP_TITLE,
                Expecting::DescriptionOrVariables => HELP_DESCRIPTION,
                Expecting::Variables => HELP_VARIABLE,
            };
            return Err(anyhow::anyhow!(
                "Unexpected EOF while {} at line {}: the last block has no variables\nHint: {}",
                self.state,
                cursor,
                help
            ));
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
    fn test_is_valid_name() {
        assert!(is_valid_name("HELLO_WORLD"));
        assert!(is_valid_name("HELLO_WORLD_42"));
        assert!(!is_valid_name("42HELLO"));
        assert!(!is_valid_name("Hello World"));
        assert!(!is_valid_name("HELLO-WORLD"));
    }

    #[test]
    fn test_is_auto_generated_variable() {
        assert!(!is_auto_generated_variable("42"));
        assert!(!is_auto_generated_variable("Hello, world!"));
        assert!(!is_auto_generated_variable("Hello, {world}!"));
        assert!(is_auto_generated_variable("Hello, {WORLD}!"));
    }

    #[test]
    fn test_is_random_variable() {
        assert!(!is_random_variable("random:42").0);
        assert_eq!(is_random_variable("random:42").1, None);
        assert!(is_random_variable("<random:42>").0);
        assert_eq!(is_random_variable("<random:42>").1, Some(42));
        assert!(is_random_variable("<random>").0);
        assert_eq!(is_random_variable("<random>").1, None);
    }

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
