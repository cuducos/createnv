use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fmt;

const DEFAULT_RANDOM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*(-_=+)";

pub struct Comment {
    pub contents: String,
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "# {}", self.contents)
    }
}

pub trait Variable {
    fn key(&self) -> String;
    fn value(&self) -> String;
    fn to_string(&self) -> String {
        format!("{}={}", self.key(), self.value())
    }
}

pub struct SimpleVariable {
    pub name: String,
    pub input: String,
}

impl Variable for SimpleVariable {
    fn key(&self) -> String {
        self.name.to_string()
    }
    fn value(&self) -> String {
        self.input.to_string()
    }
}

pub struct VariableWithDefaultValue {
    pub name: String,
    pub default: String,
    pub input: Option<String>,
}

impl Variable for VariableWithDefaultValue {
    fn key(&self) -> String {
        self.name.to_string()
    }
    fn value(&self) -> String {
        match &self.input {
            Some(value) => value.to_string(),
            None => self.default.to_string(),
        }
    }
}

pub struct AutoGeneratedVariable {
    pub name: String,
    pub pattern: String,
    pub settings: HashMap<&'static str, &'static str>,
}

impl Variable for AutoGeneratedVariable {
    fn key(&self) -> String {
        self.name.to_string()
    }
    fn value(&self) -> String {
        let mut value: String = self.pattern.to_string();
        for (k, v) in self.settings.iter() {
            let key = format!("{{{}}}", *k);
            value = value.replace(&key, *v);
        }
        value
    }
}

pub struct VariableWithRandomValue {
    pub name: String,
    pub length: Option<i32>,
}

impl Variable for VariableWithRandomValue {
    fn key(&self) -> String {
        self.name.to_string()
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

pub struct Block {
    pub title: Comment,
    pub description: Option<Comment>,
    pub variables: Vec<Box<dyn Variable>>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines: Vec<String> = vec![self.title.to_string()];
        match &self.description {
            Some(desc) => lines.push(desc.to_string()),
            None => (),
        }
        for variable in &self.variables {
            lines.push(variable.to_string());
        }
        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        let line = Comment {
            contents: "Fourty-two".to_string(),
        };
        assert_eq!(line.to_string(), "# Fourty-two")
    }

    #[test]
    fn test_variable() {
        let line = SimpleVariable {
            name: "ANSWER".to_string(),
            input: "42".to_string(),
        };
        assert_eq!(line.to_string(), "ANSWER=42")
    }

    #[test]
    fn test_empty_variable_with_default_value() {
        let line = VariableWithDefaultValue {
            name: "ANSWER".to_string(),
            default: "42".to_string(),
            input: None,
        };
        assert_eq!(line.to_string(), "ANSWER=42")
    }

    #[test]
    fn test_variable_with_default_value() {
        let line = VariableWithDefaultValue {
            name: "ANSWER".to_string(),
            default: "42".to_string(),
            input: Some("Fourty-two".to_string()),
        };
        assert_eq!(line.to_string(), "ANSWER=Fourty-two")
    }

    #[test]
    fn test_auto_generated_variable() {
        let mut settings = HashMap::new();
        settings.insert("first", "Fourty");
        settings.insert("second", "two");
        let line = AutoGeneratedVariable {
            name: "ANSWER".to_string(),
            pattern: "{first}-{second}".to_string(),
            settings,
        };
        assert_eq!(line.to_string(), "ANSWER=Fourty-two")
    }

    #[test]
    fn test_variable_with_random_value_of_fixed_length() {
        let line = VariableWithRandomValue {
            name: "ANSWER".to_string(),
            length: Some(42),
        };
        let got = line.to_string();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert_eq!(suffix.chars().count(), 42);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }

    #[test]
    fn test_variable_with_random_value() {
        let line = VariableWithRandomValue {
            name: "ANSWER".to_string(),
            length: None,
        };
        let got = line.to_string();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert!(suffix.chars().count() >= 64);
        assert!(suffix.chars().count() <= 128);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }

    #[test]
    fn test_block_with_description() {
        let title = Comment {
            contents: "42".to_string(),
        };
        let description = Some(Comment {
            contents: "Fourty-two".to_string(),
        });
        let variable1 = Box::new(SimpleVariable {
            name: "ANSWER".to_string(),
            input: "42".to_string(),
        }) as Box<dyn Variable>;
        let variable2 = Box::new(SimpleVariable {
            name: "AS_TEXT".to_string(),
            input: "fourty two".to_string(),
        }) as Box<dyn Variable>;
        let variables = vec![variable1, variable2];
        let block = Block {
            title,
            description,
            variables,
        };
        let got = block.to_string();
        assert_eq!(got, "# 42\n# Fourty-two\nANSWER=42\nAS_TEXT=fourty two")
    }
}