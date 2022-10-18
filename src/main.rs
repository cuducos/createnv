use rand::{thread_rng, Rng};
use std::collections::HashMap;

const DEFAULT_RANDOM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*(-_=+)";

struct Title {
    contents: String,
}

impl Title {
    fn as_string(&self) -> String {
        format!("# {}", self.contents)
    }
}

trait VariableLine {
    fn key(&self) -> String;
    fn value(&self) -> String;
    fn as_string(&self) -> String {
        format!("{}={}", self.key(), self.value())
    }
}

struct Variable {
    name: String,
    input: String,
}

impl VariableLine for Variable {
    fn key(&self) -> String {
        self.name.to_string()
    }
    fn value(&self) -> String {
        self.input.to_string()
    }
}

struct VariableWithDefaultValue {
    name: String,
    default: String,
    input: Option<String>,
}

impl VariableLine for VariableWithDefaultValue {
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

struct AutoGeneratedVariable {
    name: String,
    pattern: String,
    settings: HashMap<&'static str, &'static str>,
}

impl VariableLine for AutoGeneratedVariable {
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

struct VariableWithRandomValue {
    name: String,
    length: Option<i32>,
}

impl VariableLine for VariableWithRandomValue {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        let line = Title {
            contents: "Fourty-two".to_string(),
        };
        assert_eq!(line.as_string(), "# Fourty-two")
    }

    #[test]
    fn test_variable() {
        let line = Variable {
            name: "ANSWER".to_string(),
            input: "42".to_string(),
        };
        assert_eq!(line.as_string(), "ANSWER=42")
    }

    #[test]
    fn test_empty_variable_with_default_value() {
        let line = VariableWithDefaultValue {
            name: "ANSWER".to_string(),
            default: "42".to_string(),
            input: None,
        };
        assert_eq!(line.as_string(), "ANSWER=42")
    }

    #[test]
    fn test_variable_with_default_value() {
        let line = VariableWithDefaultValue {
            name: "ANSWER".to_string(),
            default: "42".to_string(),
            input: Some("Fourty-two".to_string()),
        };
        assert_eq!(line.as_string(), "ANSWER=Fourty-two")
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
        assert_eq!(line.as_string(), "ANSWER=Fourty-two")
    }

    #[test]
    fn test_variable_with_random_value_of_fixed_length() {
        let line = VariableWithRandomValue {
            name: "ANSWER".to_string(),
            length: Some(42),
        };
        let got = line.as_string();
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
        let got = line.as_string();
        let suffix = got.strip_prefix("ANSWER=").unwrap();
        assert!(suffix.chars().count() >= 64);
        assert!(suffix.chars().count() <= 128);
        let prefix = got.strip_suffix(suffix).unwrap();
        assert_eq!(prefix, "ANSWER=")
    }
}

fn main() {
    println!("Hello, world!")
}
