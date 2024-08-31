use std::fmt;
use std::{
    collections::HashMap,
    io::{stdout, BufRead, Write},
};

use anyhow::Result;
use rand::{thread_rng, Rng};

use crate::DEFAULT_ENV;

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
    fn key(&self) -> String;
    fn value(&self) -> Result<String>;
    fn as_text(&self) -> Result<String> {
        Ok(format!("{}={}", self.key(), self.value()?))
    }
}

#[derive(Clone, Debug)]
pub struct SimpleVariable {
    name: String,
    default: Option<String>,
    help: Option<String>,
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

    fn ask_for_input<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
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
            return self.ask_for_input(terminal);
        }
        if !value.is_empty() {
            self.input = Some(value.to_string());
        }
        Ok(())
    }
}

impl Variable for SimpleVariable {
    fn key(&self) -> String {
        self.name.clone()
    }
    fn value(&self) -> Result<String> {
        if let Some(input) = &self.input {
            return Ok(input.clone());
        }
        if let Some(default) = &self.default {
            return Ok(default.clone());
        }
        Err(anyhow::anyhow!("Variable {} has no value", self.name))
    }
}

#[derive(Clone, Debug)]
pub struct AutoGeneratedVariable {
    name: String,
    pattern: String,
    context: HashMap<String, String>,
}

impl AutoGeneratedVariable {
    fn new(name: &str, pattern: &str) -> Self {
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

impl Variable for AutoGeneratedVariable {
    fn key(&self) -> String {
        self.name.clone()
    }
    fn value(&self) -> Result<String> {
        let mut value: String = self.pattern.clone();
        for (k, v) in self.context.iter() {
            let key = format!("{{{}}}", *k);
            value = value.replace(&key, v);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug)]
pub struct VariableWithRandomValue {
    name: String,
    value: String,
}

impl VariableWithRandomValue {
    fn new(name: &str, length: Option<i32>) -> Self {
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

impl Variable for VariableWithRandomValue {
    fn key(&self) -> String {
        self.name.clone()
    }
    fn value(&self) -> Result<String> {
        Ok(self.value.clone())
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

    fn has_auto_generated_variables(&self) -> bool {
        self.variables
            .iter()
            .any(|v| matches!(v, VariableType::AutoGenerated(_)))
    }

    pub fn push(&mut self, variable: VariableType) -> Result<()> {
        self.variables.push(variable);
        if !self.has_auto_generated_variables() {
            return Ok(());
        }

        Ok(())
    }

    pub fn resolve<T: BufRead>(&mut self, terminal: &mut T) -> Result<()> {
        for variable in &mut self.variables {
            if let VariableType::Input(var) = variable {
                if var.input.is_none() {
                    var.ask_for_input(terminal)?;
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
                VariableType::Input(v) => context.insert(v.key(), v.value()?),
                VariableType::Random(v) => context.insert(v.key(), v.value()?),
            };
        }
        for variable in &mut self.variables {
            if let VariableType::AutoGenerated(var) = variable {
                var.load_context(&context);
            }
        }
        Ok(())
    }

    pub fn as_text(&mut self) -> Result<String> {
        let mut lines: Vec<String> = vec![self.title.to_string()];
        if let Some(desc) = &self.description {
            lines.push(desc.to_string());
        }
        for variable in &mut self.variables {
            match variable {
                VariableType::Input(var) => lines.push(var.as_text()?),
                VariableType::AutoGenerated(var) => lines.push(var.as_text()?),
                VariableType::Random(var) => lines.push(var.as_text()?),
            }
        }
        Ok(lines.join("\n"))
    }
}

// TODO: remove (only written for manual tests & debug)
pub fn model_to_text_cli() -> Result<()> {
    let variable1 = AutoGeneratedVariable::new("AUTO_GENERATED", "{ANSWER}-{DEFAULT_VALUE_ONE}");
    let variable2 = SimpleVariable::new("ANSWER", None, Some("If you read that book, you know!"));
    let variable3 = SimpleVariable::new("AS_TEXT", None, None);
    let variable4 = SimpleVariable::new("DEFAULT_VALUE_ONE", Some("default value"), None);
    let variable5 = SimpleVariable::new("DEFAULT_VALUE_TWO", Some("default"), None);
    let variable6 = VariableWithRandomValue::new("SECRET_KEY", Some(16));

    let mut block = Block::new(
        Comment::new("Here comes a new block!"),
        Some(Comment::new("And here comes a description about it.")),
    );
    block.push(VariableType::AutoGenerated(variable1))?;
    block.push(VariableType::Input(variable2))?;
    block.push(VariableType::Input(variable3))?;
    block.push(VariableType::Input(variable4))?;
    block.push(VariableType::Input(variable5))?;
    block.push(VariableType::Random(variable6))?;
    block.resolve(&mut std::io::stdin().lock())?;

    println!(
        "\nThis would be written to {}:\n\n{}",
        DEFAULT_ENV,
        block.as_text()?
    );
    Ok(())
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
        var.ask_for_input(&mut Cursor::new("42")).unwrap();
        assert_eq!(var.as_text().unwrap(), "ANSWER=42")
    }

    #[test]
    fn test_empty_variable_with_default_value() {
        let var = SimpleVariable::new("ANSWER", Some("42"), None);
        assert_eq!(var.as_text().unwrap(), "ANSWER=42")
    }

    #[test]
    fn test_variable_with_default_value_and_input() {
        let mut var = SimpleVariable::new("ANSWER", Some("42"), None);
        var.ask_for_input(&mut Cursor::new("forty two")).unwrap();
        assert_eq!(var.as_text().unwrap(), "ANSWER=forty two")
    }

    #[test]
    fn test_auto_generated_variable() {
        let mut var = AutoGeneratedVariable::new("ANSWER", "{FIRST} {SECOND}");
        let mut ctx = HashMap::new();
        ctx.insert("FIRST".to_string(), "Forty".to_string());
        ctx.insert("SECOND".to_string(), "two".to_string());
        var.load_context(&ctx);
        assert_eq!(var.as_text().unwrap(), "ANSWER=Forty two")
    }

    #[test]
    fn test_variable_with_random_value() {
        let var = VariableWithRandomValue::new("ANSWER", None);
        let got = var.as_text().unwrap();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert!(suffix.chars().count() >= 64);
        assert!(suffix.chars().count() <= 128);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }

    #[test]
    fn test_variable_with_random_value_of_fixed_length() {
        let var = VariableWithRandomValue::new("ANSWER", Some(42));
        let got = var.as_text().unwrap();
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
        variable1.ask_for_input(&mut Cursor::new("42")).unwrap();
        let variable2 = SimpleVariable::new("AS_TEXT", Some("forty two"), None);
        let mut block = Block::new(title, description);
        block.push(VariableType::Input(variable1)).unwrap();
        block.push(VariableType::Input(variable2)).unwrap();
        let got = block.as_text().unwrap();
        assert_eq!(got, "# 42\n# Forty-two\nANSWER=42\nAS_TEXT=forty two")
    }
}
