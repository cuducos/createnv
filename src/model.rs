use std::collections::HashMap;
use std::fmt;

use anyhow::Result;
use rand::{thread_rng, Rng};

const DEFAULT_RANDOM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*(-_=+)";

struct Comment {
    contents: String,
}

impl Comment {
    fn new(contents: &str) -> Self {
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
    fn value(&self) -> String;
    fn to_string(&self) -> String {
        format!("{}={}", self.key(), self.value())
    }
}

struct SimpleVariable {
    input: Option<String>,

    name: String,
    default: Option<String>,
    help: Option<String>,
}

impl SimpleVariable {
    fn new(name: &str, default: Option<&str>, help: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            default: default.map(|s| s.to_string()),
            help: help.map(|s| s.to_string()),
            input: None,
        }
    }
    fn user_input(&mut self, input: &str) {
        if let Some(help) = &self.help {
            println!("{help}");
        }
        self.input = Some(input.to_string());
    }
}

impl Variable for SimpleVariable {
    fn key(&self) -> String {
        self.name.clone()
    }
    fn value(&self) -> String {
        if let Some(input) = &self.input {
            return input.clone();
        }
        if let Some(default) = &self.default {
            return default.clone();
        }
        "".to_string() // TODO: error?
    }
}

struct AutoGeneratedVariable {
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
    fn value(&self) -> String {
        let mut value: String = self.pattern.clone();
        for (k, v) in self.context.iter() {
            let key = format!("{{{}}}", *k);
            value = value.replace(&key, v);
        }
        value
    }
}

struct VariableWithRandomValue {
    name: String,
    length: Option<i32>,
}

impl VariableWithRandomValue {
    fn new(name: &str, length: Option<i32>) -> Self {
        Self {
            name: name.to_string(),
            length,
        }
    }
}

impl Variable for VariableWithRandomValue {
    fn key(&self) -> String {
        self.name.clone()
    }
    fn value(&self) -> String {
        let mut rng = thread_rng();
        let max_chars_idx = DEFAULT_RANDOM_CHARS.chars().count();
        let mut value: String = String::from("");
        let length = match self.length {
            Some(n) => n,
            None => rng.gen_range(64..=128),
        };
        for _ in 0..length {
            let pos = rng.gen_range(0..max_chars_idx);
            value.push(DEFAULT_RANDOM_CHARS.chars().nth(pos).unwrap())
        }
        value
    }
}

enum VariableType {
    Input(SimpleVariable),
    AutoGenerated(AutoGeneratedVariable),
    Random(VariableWithRandomValue),
}

struct Block {
    title: Comment,
    description: Option<Comment>,
    variables: Vec<VariableType>,

    context: HashMap<String, String>,
}

impl Block {
    fn new(title: Comment, description: Option<Comment>, variables: Vec<VariableType>) -> Self {
        let context: HashMap<String, String> = HashMap::new();
        let has_auto_generated_variables = variables
            .iter()
            .any(|v| matches!(v, VariableType::AutoGenerated(_)));

        let mut block = Self {
            title,
            description,
            variables,
            context,
        };

        if has_auto_generated_variables {
            for variable in &block.variables {
                match variable {
                    VariableType::Input(var) => block.context.insert(var.key(), var.value()),
                    VariableType::AutoGenerated(_) => None,
                    VariableType::Random(var) => block.context.insert(var.key(), var.value()),
                };
            }

            for variable in &mut block.variables {
                if let VariableType::AutoGenerated(var) = variable {
                    var.load_context(&block.context);
                }
            }
        }

        block
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines: Vec<String> = vec![self.title.to_string()];
        match &self.description {
            Some(desc) => lines.push(desc.to_string()),
            None => (),
        }

        for variable in &self.variables {
            match variable {
                VariableType::Input(var) => lines.push(var.to_string()),
                VariableType::AutoGenerated(var) => lines.push(var.to_string()),
                VariableType::Random(var) => lines.push(var.to_string()),
            }
        }

        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        let line = Comment::new("Fourty-two");
        assert_eq!(line.to_string(), "# Fourty-two")
    }

    #[test]
    fn test_variable() {
        let mut var = SimpleVariable::new("ANSWER", None, None);
        var.user_input("42");
        assert_eq!(var.to_string(), "ANSWER=42")
    }

    #[test]
    fn test_empty_variable_with_default_value() {
        let var = SimpleVariable::new("ANSWER", Some("42"), None);
        assert_eq!(var.to_string(), "ANSWER=42")
    }

    #[test]
    fn test_variable_with_default_value_and_input() {
        let mut var = SimpleVariable::new("ANSWER", Some("42"), None);
        var.user_input("fourty two");
        assert_eq!(var.to_string(), "ANSWER=fourty two")
    }

    #[test]
    fn test_auto_generated_variable() {
        let mut var = AutoGeneratedVariable::new("ANSWER", "{FIRST} {SECOND}");
        let mut ctx = HashMap::new();
        ctx.insert("FIRST".to_string(), "Fourty".to_string());
        ctx.insert("SECOND".to_string(), "two".to_string());
        var.load_context(&ctx);
        assert_eq!(var.to_string(), "ANSWER=Fourty two")
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
        let description = Some(Comment::new("Fourty-two"));
        let mut variable1 = SimpleVariable::new("ANSWER", None, None);
        variable1.user_input("42");
        let variable2 = SimpleVariable::new("AS_TEXT", Some("fourty two"), None);
        let variables = vec![
            VariableType::Input(variable1),
            VariableType::Input(variable2),
        ];
        let block = Block::new(title, description, variables);
        let got = block.to_string();
        assert_eq!(got, "# 42\n# Fourty-two\nANSWER=42\nAS_TEXT=fourty two")
    }
}

// TODO: remove (only written for manual tests & debug)
pub fn model_to_text_cli() -> Result<()> {
    let title = Comment::new("42");
    let description = Some(Comment::new("Fourty-two"));

    let mut variable1 = SimpleVariable::new("ANSWER", None, None);
    variable1.user_input("42");
    let mut variable2 = SimpleVariable::new("AS_TEXT", None, None);
    variable2.user_input("fourty two");
    let variable3 = SimpleVariable::new("DEFAULT_VALUE_ONE", Some("default value"), None);
    let mut variable4 = SimpleVariable::new("DEFAULT_VALUE_TWO", Some("default"), None);
    variable4.user_input("custom");
    let variable5 = VariableWithRandomValue::new("SECRET_KEY", None);
    let variable6 = AutoGeneratedVariable::new("AUTO_GENERATED", "{ANSWER}-{DEFAULT_VALUE_ONE}");

    let variables = vec![
        VariableType::Input(variable1),
        VariableType::Input(variable2),
        VariableType::Input(variable3),
        VariableType::Input(variable4),
        VariableType::Random(variable5),
        VariableType::AutoGenerated(variable6),
    ];
    let block = Block::new(title, description, variables);
    println!("{block}");

    Ok(())
}
