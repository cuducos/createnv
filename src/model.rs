use std::fmt;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{stdout, BufRead, Write},
};

use anyhow::Result;
use rand::{thread_rng, Rng};

const DEFAULT_RANDOM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*(-_=+)";

#[derive(Clone)]
pub struct Comment {
    contents: String,
}

impl Comment {
    pub fn new(contents: &str) -> Self {
        Self {
            contents: contents.to_string(),
        }
    }
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "# {}", self.contents)
    }
}

trait Variable {
    fn value(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct SimpleVariable {
    pub name: String,
    pub default: Option<String>,
    pub help: Option<String>,
    input: Option<String>,
}

impl SimpleVariable {
    pub fn new(name: &str, default: Option<&str>, help: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            default: default.map(|s| s.to_string()),
            help: help.map(|s| s.to_string()),
            input: None,
        }
    }

    fn resolve<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        match (&self.help, &self.default) {
            (Some(h), Some(d)) => print!("{} [{}]: ", h, d),
            (Some(h), None) => print!("{}: ", h),
            (None, Some(d)) => print!("{} [{}]: ", self.name, d),
            (None, None) => print!("{}: ", self.name),
        };

        stdout().flush()?;
        let mut input = "".to_string();
        terminal.read_line(&mut input)?;

        let value = input.trim();
        if value.is_empty() && self.default.is_none() {
            return self.resolve(terminal);
        }
        if !value.is_empty() {
            self.input = Some(value.to_string());
        }
        Ok(())
    }
}

impl Display for SimpleVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}={}", self.name, self.value())
    }
}

impl Variable for SimpleVariable {
    fn value(&self) -> String {
        if let Some(input) = &self.input {
            return input.clone();
        }
        if let Some(default) = &self.default {
            return default.clone();
        }
        panic!(
            "Tryinyg to read the value of a {} before resolving it",
            self.name
        );
    }
}

#[derive(Clone, Debug)]
pub struct AutoGeneratedVariable {
    pub name: String,
    pattern: String,
    context: HashMap<String, String>,
}

impl AutoGeneratedVariable {
    pub fn new(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            pattern: pattern.to_string(),
            context: HashMap::new(),
        }
    }

    fn load_context(&mut self, ctx: &HashMap<String, String>) {
        for (k, v) in ctx.iter() {
            self.context.insert(k.to_string(), v.to_string());
        }
    }
}
impl Display for AutoGeneratedVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}={}", self.name, self.value())
    }
}

impl Variable for AutoGeneratedVariable {
    fn value(&self) -> String {
        let mut value: String = self.pattern.clone();
        for (k, v) in self.context.iter() {
            let key = format!("{{{}}}", *k);
            value = value.replace(&key, v);
        }
        value
    }
}

#[derive(Clone, Debug)]
pub struct VariableWithRandomValue {
    pub name: String,
    value: String,
}

impl VariableWithRandomValue {
    pub fn new(name: &str, length: Option<usize>) -> Self {
        let name = name.to_string();
        let mut rng = thread_rng();
        let max_chars_idx = DEFAULT_RANDOM_CHARS.chars().count();
        let mut value: String = String::from("");
        let length = match length {
            Some(n) => n,
            None => rng.gen_range(64..=128),
        };
        for _ in 0..length {
            let pos = rng.gen_range(0..max_chars_idx);
            value.push(DEFAULT_RANDOM_CHARS.chars().nth(pos).unwrap())
        }
        Self { name, value }
    }
}

impl Display for VariableWithRandomValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}={}", self.name, self.value())
    }
}

impl Variable for VariableWithRandomValue {
    fn value(&self) -> String {
        self.value.clone()
    }
}

#[derive(Clone, Debug)]
pub enum VariableType {
    Input(SimpleVariable),
    AutoGenerated(AutoGeneratedVariable),
    Random(VariableWithRandomValue),
}

#[derive(Clone)]
pub struct Block {
    pub title: Comment,
    pub description: Option<Comment>,
    pub variables: Vec<VariableType>,
}

impl Block {
    pub fn new(title: Comment, description: Option<Comment>) -> Self {
        Self {
            title,
            description,
            variables: vec![],
        }
    }

    fn has_auto_input_variables(&self) -> bool {
        self.variables
            .iter()
            .any(|v| matches!(v, VariableType::Input(_)))
    }

    fn has_auto_generated_variables(&self) -> bool {
        self.variables
            .iter()
            .any(|v| matches!(v, VariableType::AutoGenerated(_)))
    }

    pub fn resolve<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        if self.has_auto_input_variables() {
            println!(
                "{}",
                self.title.to_string().strip_prefix("# ").unwrap_or("")
            );
            if let Some(desc) = &self.description {
                println!("{}", desc.to_string().strip_prefix("# ").unwrap_or(""));
            }
        }
        for variable in &mut self.variables {
            if let VariableType::Input(var) = variable {
                if var.input.is_none() {
                    var.resolve(terminal)?;
                }
            }
        }
        if !self.has_auto_generated_variables() {
            return Ok(());
        }
        let mut context = HashMap::new();
        for var in &self.variables {
            match var {
                VariableType::AutoGenerated(_) => None,
                VariableType::Input(v) => context.insert(v.name.clone(), v.value()),
                VariableType::Random(v) => context.insert(v.name.clone(), v.value()),
            };
        }
        for variable in &mut self.variables {
            if let VariableType::AutoGenerated(var) = variable {
                var.load_context(&context);
            }
        }
        Ok(())
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.title)?;
        if let Some(desc) = &self.description {
            writeln!(f, "{}", desc)?;
        }
        for variable in &self.variables {
            let content = match variable {
                VariableType::Input(var) => var.to_string(),
                VariableType::AutoGenerated(var) => var.to_string(),
                VariableType::Random(var) => var.to_string(),
            };
            write!(f, "{}", content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_title() {
        let line = Comment::new("Forty-two");
        assert_eq!(line.to_string(), "# Forty-two")
    }

    #[test]
    fn test_variable() {
        let mut var = SimpleVariable::new("ANSWER", None, None);
        var.resolve(&mut Cursor::new("42")).unwrap();
        assert_eq!(format!("{}", var), "ANSWER=42")
    }

    #[test]
    fn test_empty_variable_with_default_value() {
        let var = SimpleVariable::new("ANSWER", Some("42"), None);
        assert_eq!(format!("{}", var), "ANSWER=42");
    }

    #[test]
    fn test_variable_with_default_value_and_input() {
        let mut var = SimpleVariable::new("ANSWER", Some("42"), None);
        var.resolve(&mut Cursor::new("forty two")).unwrap();
        assert_eq!(format!("{}", var), "ANSWER=forty two");
    }

    #[test]
    fn test_auto_generated_variable() {
        let mut var = AutoGeneratedVariable::new("ANSWER", "{FIRST} {SECOND}");
        let mut ctx = HashMap::new();
        ctx.insert("FIRST".to_string(), "Forty".to_string());
        ctx.insert("SECOND".to_string(), "two".to_string());
        var.load_context(&ctx);
        assert_eq!(format!("{}", var), "ANSWER=Forty two");
    }

    #[test]
    fn test_variable_with_random_value() {
        let var = VariableWithRandomValue::new("ANSWER", None);
        let got = var.to_string();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert!(suffix.chars().count() >= 64);
        assert!(suffix.chars().count() <= 128);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }

    #[test]
    fn test_variable_with_random_value_of_fixed_length() {
        let var = VariableWithRandomValue::new("ANSWER", Some(42));
        let got = var.to_string();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert_eq!(suffix.chars().count(), 42);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }

    #[test]
    fn test_block_with_description() {
        let title = Comment::new("42");
        let description = Some(Comment::new("Forty-two"));
        let mut variable1 = SimpleVariable::new("ANSWER", None, None);
        variable1.resolve(&mut Cursor::new("42")).unwrap();
        let variable2 = SimpleVariable::new("AS_TEXT", Some("forty two"), None);
        let mut block = Block::new(title, description);
        block.variables.push(VariableType::Input(variable1));
        block.variables.push(VariableType::Input(variable2));
        assert_eq!(
            block.to_string(),
            "# 42\n# Forty-two\nANSWER=42\nAS_TEXT=forty two"
        )
    }
}
